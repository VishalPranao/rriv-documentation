# Driver template

This example shows one uncalibrated sensor output, an address setting, and the methods RRIV calls. It belongs in `src/datalogger/src/drivers/example_sensor.rs`. The hardware commands still need to be filled in.

The [file generator below](#file-generation-and-naming) provides a smaller starter with dummy readings for the first build and logging check. The example here shows how settings, hardware calls, and unavailable readings fit together as the driver develops.

## Template code

Read the six Rust blocks in order as one file. Parts 4–6 belong to the same `impl SensorDriver for ExampleSensor` block; its closing brace is at the end of part 6.

### 1. Imports and driver name

```rust
use serde_json::{json, Value};

use super::types::{
    copy_config_into_partition, getters, SensorDriver, SensorDriverGeneralConfiguration,
    SENSOR_SETTINGS_PARTITION_SIZE,
};

pub const TYPE_NAME: &str = "example_sensor";
const CONFIG_VERSION: u8 = 1;
```

- `use` brings in the JSON types, driver trait, and shared configuration helpers. `copy_config_into_partition` is needed by the expanded `getters!()` macro.
- `TYPE_NAME` is the name used when configuring this driver. `CONFIG_VERSION` identifies the special-settings layout stored in EEPROM.

### 2. Device-specific settings

```rust
// An explicit, fully initialized byte layout for special settings.
#[repr(transparent)]
#[derive(Copy, Clone)]
pub struct ExampleSensorSpecialConfiguration {
    bytes: [u8; SENSOR_SETTINGS_PARTITION_SIZE],
}

impl ExampleSensorSpecialConfiguration {
    pub fn parse_from_values(value: Value) -> Result<Self, &'static str> {
        let address = Self::parse_address(&value["address"])?;
        let mut bytes = [0; SENSOR_SETTINGS_PARTITION_SIZE];
        bytes[0] = CONFIG_VERSION;
        bytes[1] = address;
        Ok(Self { bytes })
    }

    pub fn new_from_bytes(bytes: [u8; SENSOR_SETTINGS_PARTITION_SIZE]) -> Self {
        // The registry cannot return an error from this constructor.
        // Keep bytes intact; setup checks validity before hardware access.
        Self { bytes }
    }

    fn parse_address(value: &Value) -> Result<u8, &'static str> {
        let number = value.as_u64().ok_or("address must be an integer")?;
        let address = u8::try_from(number).map_err(|_| "address out of range")?;
        if !(0x08..=0x77).contains(&address) {
            return Err("unsupported address");
        }
        Ok(address)
    }

    fn address(&self) -> u8 {
        self.bytes[1]
    }

    fn is_valid(&self) -> bool {
        self.bytes[0] == CONFIG_VERSION
            && (0x08..=0x77).contains(&self.address())
            && self.bytes[2..].iter().all(|byte| *byte == 0)
    }
}
```

- The 32-byte array stores version in byte 0, address in byte 1, and zeros in the remaining bytes. `repr(transparent)` gives this wrapper the array's layout; `Copy` and `Clone` allow settings to be copied for validation.
- The registry calls `parse_from_values()` for a new JSON configuration and `new_from_bytes()` during restore. The latter cannot return an error, so `is_valid()` checks restored settings before hardware access.
- The address range is an example I2C check, not a list of addresses supported by a particular sensor. Replace it with the device's allowed addresses. `?` passes a parsing error back to the caller.

### 3. Driver state and hardware helpers

```rust
pub struct ExampleSensor {
    general_config: SensorDriverGeneralConfiguration,
    special_config: ExampleSensorSpecialConfiguration,
    measured_parameter_values: [Option<f64>; 1],
    initialized: bool,
}

impl ExampleSensor {
    pub fn new(
        general_config: SensorDriverGeneralConfiguration,
        special_config: ExampleSensorSpecialConfiguration,
    ) -> Self {
        Self {
            general_config,
            special_config,
            measured_parameter_values: [None],
            initialized: false,
        }
    }

    fn initialize_sensor(&mut self, _board: &mut dyn rriv_board::RRIVBoard) -> Result<(), ()> {
        // IMPLEMENT: documented reset/configuration/identity checks.
        // If the device needs no initialization, explicitly return Ok(()).
        Err(())
    }

    fn read_sensor(&mut self, _board: &mut dyn rriv_board::RRIVBoard) -> Result<f64, ()> {
        // IMPLEMENT: transactions, bounded waits, response validation,
        // and conversion to the documented output unit.
        Err(())
    }
}
```

- `new()` stores the settings and starts with no reading (`None`) and no successful initialization. Runtime fields are separate from the configuration saved in EEPROM.
- `initialize_sensor()` and `read_sensor()` are where the device commands go, using the supplied board interface. Both currently return errors, so this example produces no valid readings until they are implemented.

### 4. Identity, saved settings, and updates

```rust
impl SensorDriver for ExampleSensor {
    getters!();

    fn get_configuration_json(&mut self) -> Value {
        let mut id_bytes = self.general_config.id;
        let id = util::str_from_utf8(&mut id_bytes).unwrap_or_default();
        json!({
            "id": id,
            "type": TYPE_NAME,
            "address": self.special_config.address(),
            "config_valid": self.special_config.is_valid()
        })
    }

    fn update(&mut self, values: Value) -> Result<(), &'static str> {
        let mut candidate = self.special_config;
        if let Some(value) = values.get("address") {
            candidate.bytes[1] = ExampleSensorSpecialConfiguration::parse_address(value)?;
        }
        if !candidate.is_valid() {
            return Err("invalid stored configuration");
        }
        self.special_config = candidate;
        self.initialized = false;
        self.measured_parameter_values = [None];
        // The logger calls setup() after update().
        Ok(())
    }
```

- `impl SensorDriver for ExampleSensor` connects the driver to RRIV's shared interface. `getters!()` supplies the ID, type ID, and configuration-byte methods using the fields named `general_config` and `special_config`.
- `get_configuration_json()` reports the instance ID, driver type, address, and whether the settings are valid.
- `update()` validates a copy before replacing live settings, preserving an omitted address. After success it clears the reading and initialization state; the logger calls `setup()` afterward. The logger's update limitations are covered in [registry and settings](02-registry-and-settings.md).

### 5. Setup and measurement

```rust
    fn setup(&mut self, board: &mut dyn rriv_board::RRIVBoard) {
        self.measured_parameter_values = [None];
        self.initialized = false;
        if self.special_config.is_valid() {
            self.initialized = self.initialize_sensor(board).is_ok();
        }
    }

    fn take_measurement(&mut self, board: &mut dyn rriv_board::RRIVBoard) {
        self.measured_parameter_values = [None];
        if !self.initialized {
            return;
        }
        if let Ok(value) = self.read_sensor(board) {
            if value.is_finite() {
                self.measured_parameter_values[0] = Some(value);
            }
        }
    }
```

- `setup()` clears runtime state, checks the settings, and attempts initialization. It can run after creation, reboot, or a configuration update.
- Each measurement clears the previous value before reading. Only a successful, finite result becomes `Some(value)`, so a failed read cannot leave an old value appearing current. This example retries initialization only when setup runs; device-specific recovery still needs implementing.

### 6. Returning readings to the logger

```rust
    fn get_measured_parameter_count(&mut self) -> usize {
        self.measured_parameter_values.len()
    }

    fn get_measured_parameter_value(&mut self, index: usize) -> Result<f64, ()> {
        self.measured_parameter_values
            .get(index)
            .copied()
            .flatten()
            .ok_or(())
    }

    fn get_measured_parameter_identifier(&mut self, index: usize) -> [u8; 16] {
        let mut label = [0; 16];
        let text: &[u8] = if index == 0 { b"value" } else { b"invalid" };
        label[..text.len()].copy_from_slice(text);
        label
    }

    // Defaults: no actuator actions, no calibration, no GPIO requests.
}
```

- These getters expose one stored output without another hardware read. `.get(index)` checks the index, and the remaining calls turn a missing reading or invalid index into `Err(())`.
- Replace `value` with a meaningful label and document its unit. Keep labels, count, and storage in the same order when adding outputs. The trait defaults supply no GPIO requests or actuator actions; calibration remains unimplemented.

## File generation and naming

The generator uses [templates/sensor.rs](templates/sensor.rs), a dummy-data starter for task 3 of the [implementation process](04-implementation-process.md). It returns `1..10` repeatedly under the label `dummy`, reports `dummy_data: true`, and performs no hardware transactions. It requests no GPIOs, rejects configuration updates, and checks its versioned special-settings bytes before producing readings.

This checks driver creation, restore, and the logging path. Device communication, settings, and error handling can then be added using the example above. The dummy label and marker change when real readings replace the counter; failed sensor reads return unavailable values.

The [PowerShell generator](templates/new-driver.ps1) creates a file in the firmware's `src/datalogger/src/drivers/` directory. Run it from the documentation repository root, with `-FirmwareRoot` pointing to the firmware checkout. For checkouts placed beside each other:

```powershell
./firmware/driver-development/templates/new-driver.ps1 -FirmwareRoot ../rriv-firmware -Name river_temp
```

If the firmware is elsewhere, replace `../rriv-firmware` with its path.

| Item | Generated name |
| --- | --- |
| File/module | `river_temp.rs` / `river_temp` |
| Driver struct | `RiverTemp` |
| Settings struct | `RiverTempSpecialConfiguration` |
| Configured type name | `river_temp` |

Names use lowercase words separated by underscores, up to 16 ASCII bytes. The script derives the Rust type name and refuses to overwrite an existing file. It only creates the driver file; module declaration and registration remain explicit changes.

Where local PowerShell scripts are disabled, copy `firmware/driver-development/templates/sensor.rs` from this repository to `src/datalogger/src/drivers/river_temp.rs` in the firmware checkout. Replace `ExampleSensor` with `RiverTemp` and `example_sensor` with `river_temp`. The first replacement also renames the special-configuration struct.

## Registration

In `drivers/mod.rs`:

```rust
pub mod river_temp;
```

Append `crate::drivers::river_temp::TYPE_NAME` to `SENSOR_NAMES` in `registry.rs` and increase its array length. Add the constructor at the same new index:

```rust
// Example index; use the index of the newly appended name.
driver_create_functions[16] = Some(driver_create_functions!(
    crate::drivers::river_temp::RiverTemp,
    crate::drivers::river_temp::RiverTempSpecialConfiguration
));
```

For index `16`, the name array has 17 entries. Existing entries remain unchanged.

[Registry and settings](02-registry-and-settings.md) · [Next: implementation process](04-implementation-process.md)

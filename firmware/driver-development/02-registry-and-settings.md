# Registry and settings

The registry connects a configured sensor type to its Rust implementation. A type identifies the implementation; an instance ID identifies one configured device using it.

## Registry

`src/datalogger/src/registry.rs` contains two matching tables:

- `SENSOR_NAMES`: each array index is a type ID, such as `4` for `mcp_9808`.
- `get_registry()`: the same index holds constructors for that type.

`driver_create_functions!(Driver, SpecialConfiguration)` generates both construction paths:

```text
Configuration command -> parse_from_values(JSON) -> Driver::new(general, special)
EEPROM restore       -> new_from_bytes(bytes)    -> Driver::new(general, special)
```

The constructors return `Box<dyn SensorDriver>`, allowing different concrete drivers in the logger's slot array. The JSON constructor can report an error. The byte constructor cannot, so restored settings need validation before hardware access.

Registration requires the module declaration in `drivers/mod.rs`, a new name, and the matching constructor entry. **Existing type IDs stay unchanged:** EEPROM stores them, and reordering the list could load a different driver for an existing record. New names are appended. Index `0` means no match, and `12` is reserved.

Type names fit a 16-byte buffer; instance IDs fit 6 bytes. These limits concern identity, not the number of driver slots.

## Settings and related objects

| Object | Contents |
| --- | --- |
| `SensorDriverGeneralConfiguration` | Instance ID, type ID, `warmup`, and `readings_per_burst`. |
| `{DriverName}SpecialConfiguration` | Device-specific address, channel, mode, or calibration settings. |
| Driver runtime fields | Cached values, validity, and transaction/timing state. These are rebuilt at boot. |
| `DataloggerSettings` | Logger-wide mode, timing, site/deployment information, and feature switches. |
| `CalibrationPair` | One reference value and the collected raw sensor values. |
| `GpioRequest` | Dedicated pin/USART requests used for conflict checks. It doesn't configure pin modes. |

Each sensor record has 32 bytes of general settings and 32 bytes of special settings. `getters!()` copies in-memory configuration bytes into these partitions; larger data is silently truncated. The template uses an explicit special-settings byte array. Existing struct casts and the shared general-settings format still require care around alignment, padding, and compatibility with saved records.

Dynamic updates should validate a candidate before replacing live settings. The logger calls `setup()` and stores configuration afterward. It currently continues after reporting an `update()` error and lacks complete rollback for resource-changing updates, so pin reassignment needs particular care.

The general `warmup` and `readings_per_burst` fields don't supply a complete per-driver timing policy. The field measurement cycle currently uses a hardcoded burst length of 10.

Source: `registry.rs`, `drivers/types.rs`, `drivers/resources/gpio.rs`, `datalogger/settings.rs`, and `lib.rs`, under `src/datalogger/src/`.

[Traits and functions](01-traits-and-functions.md) · [Next: driver template](03-driver-template.md)

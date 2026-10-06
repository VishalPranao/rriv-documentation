# Adapting an existing sensor driver

An existing driver can save most of the register-level work, but it normally does not know about RRIV's board interface, registry, settings, or logging. Keep those responsibilities separate: the reusable library should understand the sensor, a small adapter should provide hardware access, and the RRIV driver should connect the library to `SensorDriver`.

```text
RRIV SensorDriver
    |
    | creates a temporary adapter during setup or measurement
    v
RRIV board adapter  --->  RRIVBoard
    |
    | implements the interface expected by the sensor library
    v
Reusable sensor library
```

## Record the inputs first

Before changing code, record:

- the datasheet revision and the sections covering registers, startup, timing, addresses, data format, and electrical requirements;
- the existing driver's repository, commit or release, license, `no_std` support, hardware traits, and known incomplete areas;
- the RRIV outputs, settings, GPIO requirements, error behavior, and `SensorDriver` methods needed;
- the board revision, connector, voltage, solder bridges, and test equipment;
- which behavior will stay in the reusable library and which behavior belongs in the RRIV integration.

The [planning worksheet](driver-planning.csv) includes rows for these decisions. Pin the exact source revision used during development so later changes can be compared against a known starting point.

## Choose the boundary

The reusable library should own sensor behavior such as register addresses, reset and startup sequences, response decoding, compensation, and sensor-specific errors. It should not depend on `RRIVBoard`, EEPROM, `rrivctl`, or the RRIV registry.

The RRIV driver should own the instance ID, stored settings, output cache, registry integration, and implementation of `SensorDriver`. It translates library errors into RRIV-visible state and leaves measurements unavailable after a failed read.

The adapter should contain only the hardware mapping. For an I2C device, that normally means write, write/read, and required delays. Avoid copying the sensor protocol into the adapter.

## Use a short-lived board adapter

The [SDI-12 protocol crate](https://github.com/rrivirr/rriv-firmware/tree/main/src/sdi12) and its [RRIV service adapter](https://github.com/rrivirr/rriv-firmware/blob/main/src/datalogger/src/services/sdi12_service.rs) demonstrate this pattern. `Sdi12Board<'a>` temporarily borrows `&'a mut dyn RRIVBoard` and implements the smaller `BoardForSDI12` trait. The protocol code only sees that smaller interface. The [groundwater-flow driver](https://github.com/rrivirr/rriv-firmware/blob/main/src/datalogger/src/drivers/groundwater_flow_sdi12.rs) then uses that service through the normal `SensorDriver` lifecycle.

The same approach can be used for an I2C library:

```rust
pub trait SensorBus {
    type Error;

    fn write(&mut self, address: u8, bytes: &[u8]) -> Result<(), Self::Error>;
    fn write_read(
        &mut self,
        address: u8,
        write: &[u8],
        read: &mut [u8],
    ) -> Result<(), Self::Error>;
    fn delay_us(&mut self, microseconds: u16);
}

struct RrivSensorBus<'a> {
    board: &'a mut dyn rriv_board::RRIVBoard,
}
```

`RrivSensorBus` forwards those calls to `ic2_write`, `ic2_write_read`, and `delay_us`. Another adapter can forward the same trait to `embedded-hal`, which keeps the sensor library usable outside RRIV.

The adapter should be created inside `setup()` or `take_measurement()` and dropped when the operation finishes. Do not store a reference to `RRIVBoard` inside a registry-managed driver because the board is only borrowed for the duration of each `SensorDriver` call.

This affects libraries that currently store their I2C and delay objects. Separate their persistent sensor state from the transport first:

```rust
pub struct SensorDevice {
    address: u8,
    // calibration, variant, and other state that must survive
}

impl SensorDevice {
    pub fn initialize<B: SensorBus>(&mut self, bus: &mut B) -> Result<(), SensorError> {
        // sensor protocol
        Ok(())
    }
}
```

RRIV can then keep `SensorDevice` in its driver while supplying a temporary adapter for each operation. This avoids unsafe pointers, long-lived board borrows, and repeating initialization before every measurement.

## Write the engineering plan in small pieces

For each piece of work, state:

1. **Scope:** the behavior being added and what remains for later.
2. **Files and interfaces:** which library, adapter, RRIV driver, registry, or settings files change.
3. **Data flow:** where hardware access enters, where persistent state lives, and how a result reaches RRIV outputs.
4. **Failure behavior:** timeouts, invalid responses, disconnected hardware, and recovery.
5. **Checks:** host tests with a fake bus, firmware build, hardware setup, `rrivctl` output, and reboot behavior.
6. **Completion evidence:** commands, logs, readings, and hardware configuration needed to repeat the result.

Finish and check one piece before adding the next. A useful first piece for an adapted I2C library is the interface trait, an `embedded-hal` adapter, an RRIV adapter, and an initialization test using a fake bus. Real measurements can follow after the boundary is proven.

## Branches, versions, and source history

Create a feature branch for the reusable-library change and a separate branch for RRIV integration when they are maintained in different repositories. Record the source commit in the engineering plan and keep its license and notices.

Use semantic versions for the reusable library:

- `0.1.0` for the first usable adapter/state interface;
- `0.1.1` for a compatible bug fix;
- `0.2.0` for an incompatible interface change while the API is still experimental;
- `1.0.0` once the public interface is considered stable.

Pin RRIV to a reviewed tag or commit instead of an unbounded dependency. The pull request should name the library version, hardware revision, and validation performed.

[Driver development](README.md) · [BMM350 implementation plan](examples/bmm350-implementation-plan.md)

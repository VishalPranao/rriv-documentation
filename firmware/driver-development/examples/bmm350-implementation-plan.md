# BMM350 engineering plan

This project will add the Bosch BMM350 magnetometer to RRIV. The final driver should initialize the device, read the three magnetic axes and sensor temperature, apply Bosch's factory compensation, and make the four values available to rrivctl and the datalogger.

This plan starts before implementation. Its purpose is to decide what will be built, how the existing driver code will be adapted, where the RRIV-specific boundary will sit, and how the result will be tested. The project inputs are recorded in [bmm350-driver-planning.csv](bmm350-driver-planning.csv).

## Starting inputs

Development will begin with four sources:

- The [BMM350 datasheet](https://www.bosch-sensortec.com/media/boschsensortec/downloads/datasheets/bst-bmm350-ds001.pdf) for addresses, registers, timings, data formats, and operating limits.
- Bosch's [BMM350 SensorAPI](https://github.com/boschsensortec/BMM350_SensorAPI) for the complete startup and compensation algorithms.
- The [`bmm350-rs` driver](https://github.com/wyatt-mattas/bmm350-rs/) at release `0.0.2`, commit `f9b847495164013e6bd4b60d416cafc392f650eb`, as the Rust reference.
- RRIV's `SensorDriver`, `RRIVBoard`, registry, and existing I2C drivers for the firmware conventions that the integration must follow.

The existing BMM350 Rust driver will be forked and generalized so its sensor logic works with different hardware abstraction layer (HAL) implementations. Its register map and `embedded-hal` interface provide a starting point. Before changing it, its behavior will be compared with the Bosch source, including known TODOs, incomplete compensation fields, and error paths. The fork will have one device implementation in its existing `src/device.rs`, not a second driver module.

## Scope and main decisions

The first version will use I2C and support addresses `0x14` and `0x15`, with `0x14` as the default. It will expose these values in a fixed order:

| Index | Identifier | Unit |
| --- | --- | --- |
| 0 | `mag_x` | µT |
| 1 | `mag_y` | µT |
| 2 | `mag_z` | µT |
| 3 | `temperature` | °C |

The initial operating settings will be normal mode, 100 Hz output rate, four-sample averaging, and X/Y/Z enabled. Only the I2C address will be configurable at first. Runtime changes, interrupts, I3C, forced modes, self-test, actuators, and user hard-iron or soft-iron calibration will be separate work.

Factory OTP compensation is part of the first version because it is required to turn the raw registers into meaningful measurements. Every poll or retry will have a fixed limit so a missing sensor cannot stop the logger.

## Compatibility approach

The fork will remain independent of RRIV. The existing driver owns its `embedded-hal` I2C and delay objects, while RRIV lends `&mut dyn RRIVBoard` to each `SensorDriver` operation. The fork will be adjusted to borrow a bus during setup and measurement; it will never store or import an RRIV board reference.

The bus interface will be general enough for other I2C devices, rather than named after the BMM350. Startup delays will be supplied separately:

```rust
pub trait I2CBus {
    type Error;

    fn write(&mut self, address: u8, bytes: &[u8])
        -> Result<(), Self::Error>;
    fn write_read(&mut self, address: u8, write: &[u8], read: &mut [u8])
        -> Result<(), Self::Error>;
}
```

The reusable sensor object will store persistent sensor state, including the address, variant ID, initialization status, and OTP coefficients. The bus and delay provider will be borrowed only while an operation runs. `embedded-hal` I2C and delay types can implement these small traits, but the fork's old owned-peripheral constructor will be replaced in a new minor version.

RRIV will provide a short-lived adapter inside `src/datalogger/src/drivers/bmm350.rs`:

```rust
struct BMM350I2CBus<'a> {
    board: &'a mut dyn RRIVBoard,
}
```

Its trait implementations will forward I2C and delay calls to `RRIVBoard`. It will contain no register addresses, compensation calculations, saved settings, or measurement policy.

The planned structure will be:

```text
rrivirr/bmm350-rs/             fork of the existing driver
    src/i2c_bus.rs            reusable I2CBus and Delay interfaces
    src/device.rs             BMM350 setup, reads, and compensation
    src/registers.rs          shared register map
    src/lib.rs                public exports

rriv-firmware/
    src/datalogger/Cargo.toml  Git dependency on the fork, pinned to a revision
    src/datalogger/src/drivers/bmm350.rs
                               BMM350I2CBus adapter and RRIV SensorDriver
    src/datalogger/src/drivers/mod.rs
                               module declaration
    src/datalogger/src/registry.rs
                               sensor name and constructor
```

The reusable `I2CBus` trait must be defined in the fork (or in a separate crate shared by both projects): the fork cannot import a trait from datalogger without depending on RRIV. `src/datalogger/src/adapters/i2c_bus.rs` could hold shared RRIV-side I2C helpers, but cannot be the source of the trait used by the independent fork.

## Technical decisions before implementation

These details will be settled before sensor code is changed:

- **Source strategy:** fork the existing Rust driver, replace its incomplete device logic in `src/device.rs`, and expose one borrowed-bus API. Increase the minor version because the owned-peripheral API will change.
- **Measurement format:** confirm the first register, read length, dummy bytes, byte order, signed conversion, status checks, scaling, and units from the datasheet.
- **Compensation:** identify the Bosch calculation sequence and prepare raw-data and OTP test vectors with expected results and tolerances.
- **State and recovery:** define behavior before setup, after partial setup, during repeated setup, after communication loss, and after communication returns. Check whether long polling must feed the RRIV watchdog.
- **RRIV integration:** reserve the registry ID, add workspace dependencies and modules, use an empty GPIO request, and decide how `take_measurement()` will report an error when it cannot return a `Result`.
- **Hardware:** record the exact module, voltages, connector, address selection, pin mapping, and solder bridges before applying power.

## Implementation order

1. **Audit the sources.** Create a register-and-timing checklist from the datasheet. Compare the fork's initialization and compensation code with Bosch's implementation. Identify the gaps that need work.

2. **Generalize the fork's API.** Define `I2CBus` and a delay interface, then adapt the fork's device state, typed errors, and `Measurement` to use them. Keep RRIV types out of the fork. Let `embedded-hal` peripherals use the same traits without retaining a second sensor implementation.

3. **Implement register transport.** Add bounded register reads and writes, including removal of the two dummy bytes returned by BMM350 I2C reads. Test these functions with a fake bus before adding the full startup sequence.

4. **Implement sensor setup.** Soft-reset the device, verify chip ID `0x33`, read all 32 OTP words, decode factory coefficients, power down OTP, perform magnetic reset, configure sampling and axes, and enter normal mode. PMU results and written configuration values will be read back and checked.

5. **Add the RRIV adapter and wrapper.** Pin the fork as a Git dependency in `src/datalogger/Cargo.toml`. Implement `BMM350I2CBus` alongside the RRIV `SensorDriver` in `bmm350.rs`, add its registry entry, and define the 32-byte special configuration. `setup()` will validate saved settings, create the adapter, initialize the generic device, and report the result.

6. **Implement measurements.** Read the complete X/Y/Z/temperature block in one transaction, reconstruct the signed 24-bit fields, and apply the Bosch compensation sequence. `take_measurement()` will clear old values first and publish all four values only after a complete successful read.

7. **Finish configuration behavior.** The first settings block will use byte 0 for its format version, byte 1 for the I2C address, bytes 2–5 for the fixed operating settings, and zero-fill bytes 6–31. Restored bytes will be validated before hardware access. Any later layout change will require a new configuration version.

8. **Verify the complete path.** Run host-side unit tests against a fake I2C bus to check chip ID, OTP, PMU, transaction ordering, dummy bytes, signed decoding, compensation, timeouts, and errors without hardware. Test saved settings and stale-value handling in the datalogger. Then build and flash the firmware and check configuration, registry loading, reboot restoration, output identifiers, logging, disconnect, and recovery. If the target setup supports semihosting, run repeated tests on the board with the sensor connected.

## Expected final behavior

The registry will create a BMM350 driver from RRIV's general and special settings. During `setup()`, the wrapper will temporarily lend the board to the adapter and ask the generic device to initialize. During `take_measurement()`, it will use the same boundary to request a compensated `Measurement` and copy the results into the RRIV output cache.

The RRIV measurement path should remain this small:

```rust
let result = {
    let mut bus = BMM350I2CBus::new(board);
    self.device.measure(&mut bus)
};

match result {
    Ok(value) => self.store_measurement(value),
    Err(error) => self.report_measurement_error(board, error),
}
```

All register details and compensation calculations will remain behind `self.device.measure()`. All RRIV output storage and logging will remain in the wrapper.

When setup or a read fails, the driver will leave its values unavailable and return control to the logger. It will never reuse an older reading as if it were current. rrivctl configuration output will show the selected address, fixed operating settings, initialization state, and variant ID.

The work will be ready for review when the fork's host tests and datalogger tests pass, the firmware builds and flashes, rrivctl can create and restore the sensor, all four values appear in `rrivctl watch`, rotations produce sensible axis changes, and a disconnected sensor produces a bounded error without hanging the board.

The exact breakout model, voltage, connector, pin mapping, and solder-bridge state will be filled into the worksheet before the hardware test rather than assumed from the bare-sensor datasheet.

[Adapting an existing driver](../05-adapting-existing-driver.md) · [Implementation process](../04-implementation-process.md)

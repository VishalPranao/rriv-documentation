# BMM350 implementation plan

This plan covers the first complete BMM350 sensor implementation for RRIV. The empty driver and registry entry already run. Hardware power and wiring still need to be confirmed, and the current prototype calls `RRIVBoard` directly instead of using the reusable-library adapter described below.

The main sources are the [BMM350 datasheet](https://www.bosch-sensortec.com/media/boschsensortec/downloads/datasheets/bst-bmm350-ds001.pdf), the [handling and mounting instructions](https://www.bosch-sensortec.com/media/boschsensortec/downloads/handling_soldering_mounting_instructions/bst-bmm350-hs000.pdf), the existing [`bmm350-rs` driver](https://github.com/wyatt-mattas/bmm350-rs/), and Bosch's [BMM350 SensorAPI](https://github.com/boschsensortec/BMM350_SensorAPI).

## Inputs and decisions

| Area | Input | Decision for the first scope |
| --- | --- | --- |
| Sensor | Bosch BMM350 datasheet | Three-axis magnetometer with an internal temperature measurement. |
| Outputs | BMM350 data registers | Expose `mag_x`, `mag_y`, and `mag_z` in µT and `temperature` in °C. |
| Protocol | Datasheet register map | Use I2C2. Support addresses `0x14` and `0x15`. BMM350 reads include two dummy bytes. |
| Startup | Datasheet and Bosch SensorAPI | Soft reset, verify chip ID `0x33`, read OTP, perform magnetic reset, configure the axes and sampling, then enter normal mode. Every poll must have a timeout. |
| Initial operating settings | Current RRIV prototype | 100 Hz output rate, averaging by four, X/Y/Z enabled, normal mode. Only the I2C address is initially user-selectable. |
| Compensation | Bosch OTP layout | Load and retain all factory coefficients during setup, then apply them to raw magnetic and temperature values. |
| Existing code | `wyatt-mattas/bmm350-rs` release `0.0.2`, commit `f9b847495164013e6bd4b60d416cafc392f650eb` | Reuse useful register and protocol structure, but address its incomplete compensation and validate behavior against the Bosch SensorAPI. Preserve the Apache-2.0 license. |
| RRIV interface | `SensorDriver` and `RRIVBoard` | Keep registry, EEPROM settings, output caching, and logging in the RRIV driver. Access the board through a temporary adapter. |
| Calibration | Factory and field calibration | Apply factory OTP compensation now. Leave user hard-iron/soft-iron calibration for later work. |
| Advanced features | Interrupts, self-test, I3C, forced modes | Outside the first scope. Polling over I2C is sufficient for the initial implementation. |
| Hardware | RRIV v0.4.2 and the actual BMM350 module | Confirm the module input voltage, connector orientation, and SB25/SB26 state before powering it. Record the final wiring and measured connector voltage. |

## Why an adapter is needed

The existing library uses `embedded-hal` 1.0 I2C and delay objects, but it stores those objects inside `Bmm350`. The current RRIV board crates use `embedded-hal` 0.2.7 internally and expose their own `RRIVBoard` interface to drivers. RRIV stores the board elsewhere and only lends `&mut RRIVBoard` to `setup()` and `take_measurement()`. Keeping that borrowed board inside the sensor object would not fit the RRIV lifetime or ownership model, and coupling the integration to either `embedded-hal` version would make future board changes harder.

The current RRIV prototype avoids that issue by putting register operations directly in `bmm350.rs`. It works as an initial setup implementation, but it duplicates the sensor protocol and cannot be reused by another project.

The planned design keeps persistent state separate from hardware access:

```text
BMM350 RRIV driver
    |- general and special RRIV settings
    |- cached outputs and initialization state
    `- reusable BMM350 device state
          |- address
          |- OTP compensation
          `- variant/configuration state

setup/take_measurement
    `- temporary RrivBmm350Bus(&mut RRIVBoard)
          `- implements the library's bus interface
```

The reusable library will accept a bus for each operation instead of owning it permanently. It will also provide an `embedded-hal` adapter so non-RRIV users can continue to supply normal I2C and delay implementations.

## First engineering piece: transport and initialization

### Library work

1. Branch from the recorded `bmm350-rs` revision.
2. Separate persistent device state from I2C and delay ownership.
3. Define the smallest bus interface needed by the BMM350: I2C write, combined write/read, and bounded delays.
4. Add an `embedded-hal` implementation of that interface for ordinary library users.
5. Keep the two dummy-byte handling inside the library.
6. Bring the reset, chip-ID, OTP, magnetic-reset, and configuration sequence in line with the Bosch SensorAPI.
7. Return explicit communication, chip-ID, OTP, timeout, and PMU-command errors.
8. Release the first reviewed interface as `0.1.0`; use `0.1.x` for compatible fixes and `0.2.0` for an incompatible redesign.

### RRIV work

1. Add `RrivBmm350Bus<'a>` in the RRIV integration layer. It borrows `&'a mut dyn RRIVBoard` and forwards I2C and delay operations.
2. Keep the existing type name and registry ID so stored sensor records remain compatible.
3. Keep the 32-byte special configuration versioned and validate restored bytes before hardware access.
4. In `setup()`, create the temporary adapter, initialize the reusable device state, and store the result in `initialized` and `variant_id`.
5. Leave output values unavailable when initialization fails and report the library error through RRIV debug output.

### Checks for this piece

- A fake bus verifies register ordering, dummy-byte removal, timeouts, and error propagation without hardware.
- Both the reusable library and RRIV firmware build in `no_std` mode.
- The board boots with no sensor attached and reports a bounded setup failure rather than hanging.
- With correct hardware, startup reaches normal mode and `rrivctl get sensor` reports `initialized: true`.
- Restarting the board restores the EEPROM configuration and initializes the same address again.

## Next engineering piece: measurements

1. Read one complete X/Y/Z/temperature burst through the reusable library.
2. Reconstruct the signed raw fields and check data-ready or status information required by the datasheet.
3. Apply the full Bosch OTP compensation calculation.
4. Return a typed measurement from the library.
5. In `take_measurement()`, clear old RRIV outputs first and store all four values only after a successful read.
6. Verify identifiers, units, `rrivctl watch`, storage logging, disconnect behavior, and recovery.

## Hardware information still required

Before the hardware test, add the exact BMM350 module or breakout model, its accepted input and logic voltages, the RRIV connector number, the sensor-to-connector pin mapping, the installed solder bridge, and a measured power-pin voltage. A bare BMM350 and a regulated breakout do not have the same supply requirements.

[Adapting an existing driver](../05-adapting-existing-driver.md) · [Implementation process](../04-implementation-process.md)

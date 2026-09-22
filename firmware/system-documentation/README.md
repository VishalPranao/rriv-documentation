# System documentation

The firmware separates board hardware from datalogger behavior. Sensor drivers sit between them: each driver understands one device's protocol, while the datalogger manages its configuration and measurements.

## Project structure

| Location | Role |
| --- | --- |
| `board/app/` | Firmware entry point, initialization, and main loop. |
| `board/rriv_board/` | Shared `RRIVBoard` hardware interface. |
| `board/rriv_board_0_4_2/` | Board-specific implementation of that interface. |
| `src/datalogger/` | Driver management, timing, command handling, logging, and telemetry. |
| `src/datalogger/src/drivers/` | Sensor implementations and the `SensorDriver` trait. |
| `src/datalogger/src/registry.rs` | Sensor type names and constructors. |
| `src/datalogger/src/services/` | Command, USART, and SDI-12 services. |
| `src/control_interface/` | Command recognition and command types. |
| `src/util/`, `src/sdi12/` | Shared helpers and SDI-12 support. |

`board/` and `src/` are separate Cargo workspaces. The repository root has no `Cargo.toml`.

## Startup and measurements

`main.rs` initializes the board, creates a `DataLogger`, and calls its setup method. The logger restores settings from EEPROM, creates the configured drivers through the registry, checks their resource requests, and calls each driver's `setup()`.

The main loop then calls the board and datalogger loop methods repeatedly:

```text
Main loop
  -> board services
  -> datalogger commands and timing
  -> driver.take_measurement(board), when a measurement is due
  -> driver stores the result
  -> logger retrieves values and labels for output
```

Drivers are called sequentially. A long delay in one driver delays other work too. Measurement frequency depends on the logger's operating mode; getters return cached readings rather than starting another transaction.

The logger stores drivers in `Option<Box<dyn SensorDriver>>` slots. `None` is an empty slot; `Some` owns a configured driver. JSON commands create or update instances, while EEPROM allows them to be restored after reboot. Instance count is limited by the EEPROM configuration; the default feature provides 12 slots.

The loop also calls `update_actuators()`, which has an empty default implementation. Actuator implementation is outside the sensor work described here.

Source: `board/app/src/main.rs`, `src/datalogger/src/lib.rs`, and `board/rriv_board/src/lib.rs`.

[Technical background](../technical-background/README.md) · [Driver development](../driver-development/README.md)

# Traits and functions

`SensorDriver`, defined in `src/datalogger/src/drivers/types.rs`, is the interface between a driver and the datalogger. Each sensor supplies its own implementation of the required methods.

## Required methods

| Method | Role |
| --- | --- |
| `setup(board)` | Prepares runtime state and hardware after creation, restore, or update. |
| `take_measurement(board)` | Communicates with the sensor and stores a result. |
| `get_measured_parameter_count()` | Returns the number of exposed output values. |
| `get_measured_parameter_value(index)` | Returns a cached value as `Ok(value)`, or `Err(())` when unavailable. |
| `get_measured_parameter_identifier(index)` | Returns the output label as `[u8; 16]`. |
| `get_configuration_json()` | Describes the instance and its settings for the command interface. |
| `get_configuration_bytes(storage)` | Writes settings into the EEPROM record buffer. |
| `update(values)` | Applies settings changes, returning a parsing/validation error when needed. |
| `get_id()` / `get_type_id()` | Returns the instance ID / registry type ID. |

A declaration ending in `;` needs an implementation. A method with a body already has a default that can be kept or overridden. Signatures must match the trait, including mutable access where declared.

```rust
fn get_measured_parameter_value(&mut self, index: usize) -> Result<f64, ()>;
```

This method borrows the driver mutably and returns either a floating-point value or an error without further details. The index refers to an output, not necessarily a separate physical quantity: one temperature measurement may produce both `Traw` and `Tcal`.

`take_measurement()` and `setup()` return no `Result`. A private helper can return errors, while the driver stores validity and exposes unavailable measurements through its getter. Repeated setup and failed reads need defined behavior.

## Optional methods and helpers

| Method/helper | Default or purpose |
| --- | --- |
| `get_requested_gpios()` | Requests no resources by default. Overrides declare dedicated GPIO/USART use. |
| `fit(pairs)` | Returns `Err(())` by default; supported drivers calculate calibration coefficients. |
| `clear_calibration()` | Does nothing by default; supported drivers reset calibration. |
| `update_actuators(board)` | Does nothing by default. Actuator implementation is deferred. |
| `getters!()` | Generates ID getters and configuration-byte copying. |
| `single_raw_or_cal_parameter_identifiers()` | Produces `raw`/`cal`, with an optional prefix such as `T`. |

The getters macro expects fields named `general_config` and `special_config`, with `copy_config_into_partition` in scope. Hardware calls use `&mut dyn RRIVBoard`, a temporary borrow of the shared board interface.

`generic_analog.rs` shows ADC reads and a raw/calibrated pair; `mcp9808.rs` shows an I2C temperature transaction. Their structure is useful, though both leave `update()` unimplemented and MCP9808's failure paths invalidate only its first output.

[Driver development](README.md) · [Next: registry and settings](02-registry-and-settings.md)

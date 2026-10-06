# Driver implementation process

First work out what the sensor needs and how it connects to the board. Then get a driver showing dummy readings and add setup, real measurements, settings, and calibration one piece at a time. Rebuild and flash after code changes before checking the result with rrivctl.

The steps below follow that order. Driver filenames refer to `src/datalogger/src/drivers/` in the firmware repository. Actuator implementation is left for separate work.

## 1. Work out what the driver needs

Before writing code, open the sensor datasheet and the [trait reference](01-traits-and-functions.md). Work out what the device needs and which driver methods will handle it:

- **Role:** what does the sensor measure? Does it also control something, such as a heater or switch? Record any actuator requirements for later.
- **Outputs:** how many values should RRIV log, and in what units? One temperature measurement could provide one value, or two outputs if both raw and calibrated temperatures are needed.
- **Connection:** which protocol does it use—I2C, SPI, I2S, analog input, USART, or OneWire? Does it also need dedicated GPIOs or PWM?
- **Calibration:** does the device support calibration, and will this driver implement it? If so, decide what reference measurements are needed; this affects both output layout and saved settings.
- **Composition:** can one driver handle the device, or will it contain other drivers? For a compound driver, identify the child drivers and who will coordinate their hardware access.
- **Settings:** what should be configurable, such as address, channel, or operating mode? Write down each setting's default, allowed values, and whether it can change while running.

Put the answers and relevant datasheet sections in the [planning worksheet](driver-planning.csv), or use its questions as notebook headings. For an analog sensor, first check whether `generic_analog.rs` already provides the required behavior.

Then compare the hardware needs with `RRIVBoard` in `board/rriv_board/src/lib.rs`. It exposes I2C, ADC, GPIO, USART, PWM, and OneWire operations, but no general SPI or I2S interface. Identify missing board support now because it adds work beyond the driver itself.

If usable driver code already exists, record its repository, exact version or commit, license, hardware interface, and incomplete features. Decide which sensor logic can remain reusable and which RRIV-specific work belongs in the integration. Follow [adapting an existing driver](05-adapting-existing-driver.md) when a small board adapter can connect the two without copying the whole implementation into the RRIV driver.

At this point, the plan should name the outputs, settings, hardware resources, and trait methods to implement.

## 2. Prepare and record the hardware connection

Use the sensor datasheet and the schematic for the actual board revision to choose the connector and pins. Check supply voltage, signal voltage, ground, and any required pull-ups or interface circuitry. These choices also determine the address, channel, or pin numbers used in the code.

With power disconnected, connect the sensor and make any required solder-bridge changes. Check the board's power routing carefully where a pin can supply 3.3 V or 5 V; the required bridge setting depends on the board revision.

Record the sensor-pin-to-board-pin mapping, connector name, voltage, and original/final bridge positions in the worksheet. Verify the wiring against those notes before powering the board. Someone repeating the test should be able to reproduce the connection from this record.

### I2C sensors

Check whether the voltage printed on the sensor board applies to the sensor chip or to a breakout board with its own regulator and level shifting. Then map power, ground, SDA, and SCL from the sensor board to the RRIV connector. Also record the sensor address and any address-select jumper position.

On RRIV v0.4.2, SB25 supplies 5 V to the primary I2C power pins and SB26 supplies 3.3 V. They are mutually exclusive and must not both be installed. The 5 V rail is not powered by default, and J6 has additional bridge options, so check the [board configuration](../../hardware/manual/configuration.md) before changing a bridge. With the sensor disconnected, measure the connector's power pin relative to ground and confirm it matches the sensor board's input requirement.

If the board starts normally without the sensor but restarts after it is connected, disconnect it and recheck the voltage, connector orientation, pinout, and SDA/SCL wiring. A shorted supply or an I2C device holding a bus line low can stop startup at the `unhang I2C2 if hung` stage. Resolve that electrical problem before debugging the driver.

## 3. Get the empty driver running

Use the **dummy-data generator** on the [template page](03-driver-template.md#file-generation-and-naming) for this first check. The longer example on that page contains hardware placeholders that return errors until implemented.

Choose a name such as `river_temp`, then make the three integration changes described there:

1. Generate `river_temp.rs` in the drivers folder.
2. Add `pub mod river_temp;` to `drivers/mod.rs`.
3. Append the name and matching constructor entry in `src/datalogger/src/registry.rs`. Keep existing type IDs unchanged because saved configurations depend on them.

Build using the commands in [technical background](../technical-background/README.md#building-the-firmware). Then flash and run on the development board. The VS Code configuration `Datalogger: Build & Debug` in `board/.vscode/launch.json` provides a build/flash/debug path; its toolchain, chip, and probe setup must match the development environment.

Create a local `river_temp.json` containing:

```json
{"type":"river_temp"}
```

With rrivctl installed and the board connected, configure a new instance and watch it:

```sh
rrivctl set datalogger mode interactive
rrivctl set sensor test01 -f river_temp.json
rrivctl get sensor test01
rrivctl watch
```

The commands are described in the [rrivctl manual](https://github.com/rrivirr/rriv-documentation/blob/main/rrivctl/rrivctl-manual.md). Exit watch with Ctrl+C before sending another command.

Look for `test01_dummy` repeating `1..10` and `dummy_data: true` in the configuration. Restart the board and confirm the instance returns. This establishes that creation, restore, and serial output work; it does not test the sensor connection. Check storage logging separately when enabled. If this stage fails, resolve registration, build, or logger-mode issues before adding device commands.

## 4. Add the setup the sensor actually needs

Read the datasheet's startup sequence. Some sensors can be read immediately; others need a reset, mode selection, or conversion startup. Put those operations in `setup()`, using the board interface. If the device needs no initialization, setup may only need to reset cached values and runtime state.

If dedicated GPIOs or USART are needed, override `get_requested_gpios()`. In `resources/gpio.rs`, `GpioRequest::none()`, `use_pin()`, and `use_usart()` show how requests are built. The logger checks ownership before setup; configure pin modes separately through the board. Check the available pins before choosing one: the allocator rejects GPIO 3 and 4, and USART occupies GPIO 7 and 8. A sensor using only I2C can keep the default empty request.

Use fixed, documented settings for the first connection; making them configurable comes in step 7. Handle required warmup and bounded retries explicitly—the general `warmup` field does not schedule them. The [template's setup example](03-driver-template.md#5-setup-and-measurement) shows how to record initialization success.

Try both a cold boot and repeated setup. If initialization fails, keep real readings unavailable. Dummy output can still help check the logger while developing setup, but use a device response or debugger observation to confirm hardware initialization.

## 5. Read one real measurement

Now replace the dummy counter in `take_measurement()` with the smallest complete read: send the device command, wait or poll as required, read the response, and convert it to the chosen unit. Start with one quantity using the template's existing storage.

For an I2C example, read `mcp9808.rs`; for an ADC input, read `generic_analog.rs`. Use their structure, then work out the actual commands and decoding from the new sensor's datasheet. The board methods are spelled `ic2_read`, `ic2_write`, and `ic2_write_read`; choose separate or combined transactions according to the sensor protocol.

A private `read_sensor()` returning `Result` keeps transaction errors manageable because `take_measurement()` itself returns no result. Clear the previous reading first, and store a value only after checking the response, sign, byte order, scale, and any status/checksum bits. Set a limit on polling or retries so a disconnected sensor cannot stall the logger. Replace the `dummy` label and `dummy_data` marker when real measurements take over.

Check decoding with known response bytes, then compare a hardware reading with a reference. Test a failed or disconnected read too: the output should become unavailable, with no old or dummy value substituted. Verify recovery when communication returns.

## 6. Connect the readings to watch and logging

Once one real reading works, expand the output storage to match the plan. Give each output a fixed index, label, unit, and validity state. For example, temperature with calibration can use index 0 for `Traw` and index 1 for `Tcal`; step 8 adds the calibration calculation.

Keep these pieces consistent:

- **Runtime storage** holds the latest values; `Option<f64>` is one way to represent an unavailable reading.
- **`get_measured_parameter_count()`** returns the number of exposed values.
- **`get_measured_parameter_value(index)`** returns a stored value without reading the sensor again, or `Err(())` for a missing value or invalid index.
- **`get_measured_parameter_identifier(index)`** returns its label in the 16-byte buffer. The logger adds the instance ID, producing a column such as `test01_Traw`.

Invalidate every output affected by a failed read, including calibrated values. MCP9808's existing failure path clears only its first output, so use the template's explicit validity handling when adapting it. Compare each watch column with the planned index and unit, including after reboot.

## 7. Make the settings configurable

Move the fixed choices from step 4 into `{DriverName}SpecialConfiguration`. Start with the settings identified in the plan, such as address or channel. The [template's address setting](03-driver-template.md#2-device-specific-settings) shows the full path:

1. Parse and validate new settings in `parse_from_values()`.
2. Report them through `get_configuration_json()`.
3. In `update()`, copy the current settings, apply the supplied fields, validate the candidate, then replace the live settings. Preserve fields left out of the command.
4. Store them through `get_configuration_bytes()`—supplied by `getters!()`—and restore them through `new_from_bytes()`.

Keep the special-settings layout within 32 bytes and validate restored bytes before hardware access. Document the byte layout/version as described in [registry and settings](02-registry-and-settings.md). Changes from the dummy template's layout need an explicit restore policy; incompatible saved bytes should be rejected or migrated.

For an update through rrivctl, use the same instance ID and a JSON file containing only the changed settings; omit `type`. The logger calls setup afterward and currently continues even after an update error, so rejected changes must leave settings intact. Pin reassignment also needs work in the logger's rollback path.

Try a valid update, an invalid value, and a partial update. Read the settings back, check that behavior matches them, and restart the board to verify persistence.

## 8. Add calibration if the sensor needs it

If calibration is outside the plan, keep the default methods and record that choice. Otherwise, choose the fitting method before implementing `fit()`: a constant offset needs one reference point; a linear slope and intercept need two distinct raw readings. `mcp9808.rs` and `generic_analog.rs` show those two approaches.

Implement `fit()` to validate the reference data and calculate coefficients, store those coefficients in special settings, and apply them when producing calibrated outputs. Define the reset coefficients in `clear_calibration()`. Reject bad point counts, non-finite values, and zero denominators before changing coefficients.

Check the collection path in `src/datalogger/src/lib.rs` before wiring this up. It expects `raw0, cal0, raw1, cal1, ...`, keeps at most two reference points with space for 10 raw values, and substitutes zero for failed readings. It also clears calibration before fitting. These behaviors mean a successful formula alone is not enough: verify that the collected measurements are valid, and change the collection path if the planned calibration cannot use it.

For each known reference, let the sensor settle and collect a point with `rrivctl calibrate sensor test01 point <reference>`. Inspect the points with `rrivctl calibrate sensor test01 list`, then run `rrivctl calibrate sensor test01 fit`. Check the corrected readings and repeat after reboot. Successful fits are saved; the standalone clear command currently does not save the cleared state, so test that behavior explicitly. Command details are in the [rrivctl calibration reference](https://github.com/rrivirr/rriv-documentation/blob/main/rrivctl/rrivctl-manual.md#calibrate-sensor).

## 9. Handle the remaining design work

Return to the requirements set aside during planning. Leave `update_actuators()` at its default for this sensor-only implementation, and keep any actuator requirements as a separate task.

If the driver needs composition, read `ring_w_mux.rs`, which owns several MCP9808 drivers. Decide which driver owns shared resources, calls child setup/measurement methods, and combines the outputs. Test a child failure as well as the combined readings. Protocol-specific templates that check the physical connection are also future work; the current generator only exercises the firmware integration.

Finally, run the sensor alongside another configured driver. Check normal readings, failure/recovery, settings restoration, and storage output when enabled. Record the firmware revision, wiring, configuration, and results in the worksheet so the next developer can repeat the same test.

[Driver template](03-driver-template.md) · [Driver development](README.md)

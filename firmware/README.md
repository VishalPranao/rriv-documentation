# Firmware

RRIV firmware is the software running on the datalogger board. It manages connected sensors, takes measurements, and makes the results available through serial output, storage, and configured telemetry. It also handles configuration commands and saves settings so the same sensor setup can be restored after a restart.

The code separates device-specific behavior from the rest of the logger. Sensor drivers handle communication and data conversion, the board layer provides hardware access, and the datalogger coordinates when readings happen and where they go. A new sensor usually connects to this structure through the common driver interface.

The documentation is organized into three sections, covering the development background, the system itself, and sensor implementation.

## [Technical background](technical-background/README.md)

An introduction to Rust, the benefits it provides for firmware, and its relationship to C/C++. It also covers RRIV's embedded environment and development tools. Language syntax and examples are linked to the Rust manuals, while the local notes focus on the concepts relevant to this project.

## [System documentation](system-documentation/README.md)

The structure of the firmware repository and how its parts work together. This section covers the board and application code, datalogger, drivers, registry, and shared services. It follows startup and the measurement loop, including how configured drivers are restored and how readings reach the outputs.

## [Driver development](driver-development/README.md)

The reference material and implementation process for adding a sensor. It is split into four documents:

1. [Traits and functions](driver-development/01-traits-and-functions.md): the driver interface and the role of each method.
2. [Registry and settings](driver-development/02-registry-and-settings.md): driver creation, identity, configuration objects, and persistence.
3. [Driver template](driver-development/03-driver-template.md): file generation, naming, registration, and the initial dummy-data implementation.
4. [Implementation process](driver-development/04-implementation-process.md): the nine tasks, from planning and wiring through real measurements, settings, and calibration, with checks at each stage.

The driver work currently focuses on sensors. Actuator implementation is deferred, with any related requirements recorded during planning.

Source paths in these pages refer to the [RRIV firmware repository](https://github.com/rrivirr/rriv-firmware).

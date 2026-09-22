# Driver development

A sensor driver implements the common interface, holds device settings, and converts hardware responses into named measurements. These four documents separate the reference material from the order of implementation:

1. [Traits and functions](01-traits-and-functions.md) — the interface and each method's role.
2. [Registry and settings](02-registry-and-settings.md) — driver creation, identity, configuration, and storage.
3. [Driver template](03-driver-template.md) — the initial implementation, naming, and file generation.
4. [Implementation process](04-implementation-process.md) — the nine tasks and the checks at each stage.

The initial template produces dummy data so registration and logging can be checked before sensor communication is implemented. Actuator work is deferred; resource requirements and compound drivers are still recorded during planning.

The [planning worksheet](driver-planning.csv) opens in a spreadsheet and can also be used as notebook headings.

[Firmware](../README.md)

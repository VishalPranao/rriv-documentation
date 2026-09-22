# Technical background

Rust is a compiled systems programming language used for software that needs direct control over memory and hardware. It serves a similar role to C and C++, with additional compiler checks around memory ownership and access. Memory management does not require a garbage collector. [Rust language overview](https://rust-lang.org/)

## What Rust provides for firmware

- **Memory safety:** ownership and borrowing rules catch problems such as using freed data or allowing conflicting mutable references in safe code.
- **Control over resources:** native compilation and support for fixed-size storage suit devices with limited RAM and processing capacity.
- **Explicit error handling:** `Result` and `Option` distinguish successful values, failures, and missing data. In a sensor driver, these distinctions help keep an unavailable measurement separate from a valid reading.
- **Common interfaces:** traits let different devices provide the same operations. RRIV uses `SensorDriver` so the logger can work with several sensor implementations through one interface.
- **Development tools:** Cargo, compiler diagnostics, formatting, and editor support make building and maintaining the code more consistent.

These features help catch software mistakes before firmware reaches the board. Hardware timing, calibration formulas, and the assumptions behind `unsafe` code still need validation.

## Relationship to C/C++

Structs, methods, generics, and automatic cleanup have familiar C++ counterparts. Rust traits play a role similar to interfaces, while ownership builds on ideas familiar from RAII and smart pointers.

The main change is that the compiler checks when data can be moved, borrowed, or changed. Access that might be left to convention in C++ becomes part of the Rust type and borrowing rules. The detailed syntax and examples are covered in the references below.

## Common C++ concepts in Rust

These are useful connections when reading the driver code, though they are not exact equivalents.

| C++ concept | Rust counterpart |
| --- | --- |
| Functions and member functions | `fn` declares a function. Inside an `impl` block, a function with a `self` parameter is a method. |
| Classes with data and methods | A `struct` holds data; `impl TypeName` defines its methods and associated functions. Rust has no `class` keyword. |
| `this` | `self` refers to the current instance. `&self` borrows it for reading; `&mut self` borrows it for changes. |
| Variable declarations | `let` declares a binding that is immutable by default; `let mut` allows changes. This is different from C++, where ordinary variables are mutable by default. |
| References | `&T` and `&mut T` borrow a value. The compiler checks their lifetimes and prevents conflicting access. |
| Templates | Generics use parameters such as `<T>`. Trait bounds describe the operations a generic type must support. |
| Abstract interfaces and virtual methods | A `trait` defines shared behavior; `impl SensorDriver for MySensor` supplies it. `dyn SensorDriver` allows runtime dispatch to different driver types. |
| Constructors and destructors | `new()` is a naming convention for an ordinary associated function. Values are cleaned up automatically; `Drop` provides custom cleanup, similar to a destructor. |

Some familiar C++ features work differently or are absent:

- **Class inheritance:** Rust has no class inheritance. Shared behavior uses traits, and a struct can contain other structs to reuse their functionality.
- **Function overloading and default arguments:** Rust does not overload functions by parameter types or provide default arguments. Separate names, generics, or configuration structs cover common cases.
- **Exceptions:** Recoverable errors use `Result` and often `?` to return an error to the caller. `panic!` is separate and is not a replacement for routine error handling.
- **Header files:** Rust uses modules and `pub` visibility instead of separate `.h` declarations and `.cpp` implementations.

## Rust in RRIV

The datalogger uses `no_std` outside tests, allowing it to operate without the usual operating-system services. Basic types and operations come from `core`; heap-backed types are available through `alloc`. Hardware access is provided by the board layer. This is the kind of environment described in the [Embedded Rust Book's `no_std` section](https://docs.rust-embedded.org/book/intro/no-std.html).

## Language references

- [The Rust Book](https://doc.rust-lang.org/book/): language fundamentals, types, traits, and error handling.
- [Ownership and borrowing](https://doc.rust-lang.org/book/ch04-00-understanding-ownership.html): the main conceptual difference for developers coming from C/C++.
- [Rust by Example](https://doc.rust-lang.org/rust-by-example/): short, runnable examples of syntax and language features.
- [The Embedded Rust Book](https://docs.rust-embedded.org/book/intro/index.html): microcontrollers, embedded development, and hardware access.

## Development tools

- **[Git](https://git-scm.com/install/)** downloads the repository and tracks source changes.
- **[VS Code](https://code.visualstudio.com/)** provides the editor and project workspace.
- **rustup** installs Rust toolchains and targets.
- **Cargo** checks, builds, and tests crates.
- **rust-analyzer** provides editor navigation and diagnostics.
- **probe-rs** flashes and debugs the board.
- **rrivctl** configures the running firmware and displays measurements.

### Repository access and cloning

The source is hosted at [rrivirr/rriv-firmware](https://github.com/rrivirr/rriv-firmware). It is a public repository, so reading and cloning it does not require write access. Direct pushes require access granted by a maintainer to the contributor's GitHub account. Contributions can also be submitted through a fork and pull request.

After installing Git, open PowerShell or another terminal in the directory where the project should be stored:

```sh
git clone https://github.com/rrivirr/rriv-firmware.git
cd rriv-firmware
```

This creates a local `rriv-firmware` folder containing the source and Git history. The commands below assume this folder is the working directory.

### Opening the project in VS Code

The repository includes `rriv-rust.code-workspace`, which opens the `board` and `src` folders together:

```sh
code rriv-rust.code-workspace
```

If the `code` command is unavailable, launch VS Code and choose **File > Open Workspace from File**, then select `rriv-rust.code-workspace` inside the cloned folder. Install the **rust-analyzer** extension from the Extensions panel for Rust navigation and diagnostics. VS Code's [workspace documentation](https://code.visualstudio.com/docs/editing/workspaces/multi-root-workspaces) covers this opening method.

**Terminal > New Terminal** opens an integrated terminal. Its working directory may be `board` or `src` in this workspace; the build commands below start from the repository root.

### Building the firmware

The application uses nightly Rust and the `thumbv7m-none-eabi` target. Setup and build commands, starting at the firmware repository root:

```sh
rustup toolchain install nightly
rustup target add thumbv7m-none-eabi --toolchain nightly
cd board
cargo +nightly check -p app
cargo +nightly build --release -p app
```

Running from `board/` picks up `.cargo/config.toml`, including target and linker settings. Flashing uses the established board/probe configuration; its chip selection must match the connected board.

Source: `README.md`, `board/.cargo/config.toml`, `.github/workflows/build-binary.yaml`, and `board/app/src/main.rs`.

[Firmware](../README.md) · [System documentation](../system-documentation/README.md)

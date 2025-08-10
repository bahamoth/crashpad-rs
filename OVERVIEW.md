<!-- DOCGUIDE HEADER
Version: 1.0
Generated: 2025-08-06
Project Type: Rust
Repository: https://github.com/bahamoth/crashpad-rs.git
Last Updated: 2025-08-06
Update Command: claude commands/scaffold/overview.md
-->

# crashpad-rs – Overview

Rust bindings for Google Crashpad crash reporting system with comprehensive cross-platform support.

## Repository Structure

```
crashpad-rs/
├── crashpad-sys/           # Low-level FFI bindings to Crashpad
│   ├── build.rs           # Build script for native compilation
│   ├── build/             # Build system modules
│   │   ├── config.rs      # Build configuration
│   │   ├── phases.rs      # Build phases orchestration
│   │   └── tools.rs       # Binary tool management (GN/Ninja)
│   ├── crashpad_wrapper.cc # C++ wrapper for Crashpad API
│   ├── wrapper.h          # Header for FFI bindings
│   └── src/               # Generated FFI bindings
├── crashpad/              # Safe Rust API wrapper
│   ├── src/
│   │   ├── client.rs     # Main client interface
│   │   ├── config.rs     # Configuration builder
│   │   └── lib.rs        # Library entry point
│   ├── examples/         # Usage examples
│   └── tests/            # Integration tests
├── xtask/                 # Development task runner
├── third_party/           # Git submodules for dependencies
│   ├── crashpad/         # Crashpad source
│   ├── mini_chromium/    # Mini Chromium library
│   ├── googletest/       # Google Test framework
│   ├── zlib/             # Compression library
│   ├── libfuzzer/        # Fuzzing library
│   ├── edo/              # EDO library
│   └── lss/              # Linux Syscall Support
├── CLAUDE.md             # AI assistant guidelines
├── PRD.md                # Product requirements document
├── ARCHITECTURE.md      # System design documentation
└── CONVENTIONS.md        # Code, build, and deployment standards
```

## Getting Started

### Prerequisites

- Rust 1.70+
- C/C++ compiler (gcc/clang)
- libclang (for bindgen)
- Platform SDKs:
  - Android: NDK r22+ with symlinks configured
  - iOS: Xcode with iOS SDK
  - Windows: Visual Studio or MinGW

### Quick Start

1. Clone the repository:
   ```bash
   git clone https://github.com/bahamoth/crashpad-rs
   cd crashpad-rs
   ```

2. Initialize submodules and build:
   ```bash
   git submodule update --init --recursive
   cargo build --package crashpad-sys
   cargo build --package crashpad
   ```

3. Run the example:
   ```bash
   cargo run --example crashpad_test_cli
   ```

### Cross-Compilation Quick Start

#### Android (using cargo-ndk)
```bash
cargo install cargo-ndk
cargo ndk -t arm64-v8a build --package crashpad-sys    # ARM64
cargo ndk -t armeabi-v7a build --package crashpad-sys  # ARMv7
```

#### iOS
```bash
rustup target add aarch64-apple-ios
cargo build --target aarch64-apple-ios                 # Device
cargo build --target aarch64-apple-ios-sim             # Simulator
```

For detailed cross-compilation instructions, see [Conventions](./CONVENTIONS.md#cross-compilation).

## Core Concepts

### Two-Crate Architecture
- **crashpad-sys**: Low-level FFI bindings using bindgen, handles native compilation
- **crashpad**: Safe Rust API providing ergonomic interface with error handling

### Git Submodule Dependencies
The project uses Git submodules for dependency management, following the exact hierarchy from Crashpad's DEPS file. All dependencies are in `third_party/` and must be initialized with `git submodule update --init --recursive`.

### Cross-Platform Handler
- Desktop platforms use external `crashpad_handler` executable
- iOS uses in-process handler (no separate executable needed)
- Handler path resolution follows platform conventions

### Build System
Uses streamlined native build tools:
- Git submodules for dependency management
- GN (downloaded automatically) for build file generation  
- Ninja (downloaded automatically) for compilation
- No Python or depot_tools required

## Development

- `cargo xtask build` - Build the project
- `cargo xtask test` - Run test suite
- `cargo xtask dist` - Create distribution package
- `cargo xtask clean` - Clean all artifacts
- `make clean` - Clean native build artifacts only

## Integration

### Quick Integration

1. **Build for production**:
   ```bash
   cargo xtask dist
   ```

2. **Package structure**:
   ```
   my-app/
   ├── my-app              # Your application
   ├── crashpad_handler    # Handler executable (required)
   └── lib/                # Any required libraries
   ```

3. **Handler location options**:
   - Same directory as application (recommended)
   - Set via `CRASHPAD_HANDLER` environment variable
   - Configure programmatically in code

For detailed integration procedures, see [Conventions](./CONVENTIONS.md#integration).

## Related Documentation

- [Architecture](./ARCHITECTURE.md) - System design and technical decisions
- [Conventions](./CONVENTIONS.md) - Code style, build, and deployment standards
- [PRD](./PRD.md) - Product requirements and roadmap
- [Tasks](./TASKS.md) - Current development tasks

## Requirement Tracking

This project uses a structured ID system for requirements and tasks.
See [PRD.md#id-scheme](./PRD.md#id-scheme) for details on:
- R-### (Requirements)
- G-### (Goals)
- T-### (Tasks)
- D-YYYY-MM-DD-## (Decisions)
- Q-### (Questions)
# crashpad-rs

Rust bindings for Google Crashpad crash reporting system.

## ⚠️ Important: Android Build Requirements

**Android builds require creating symlinks for NDK r22+ compatibility:**

NDK r22 and later removed standalone toolchain binaries (e.g., `aarch64-linux-android-ar`) in favor of LLVM tools. You must create symlinks to ensure compatibility:

```bash
# Navigate to your NDK toolchain directory
cd $ANDROID_NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin

# Create symlinks for each target architecture
ln -sf llvm-ar aarch64-linux-android-ar
ln -sf llvm-ar arm-linux-androideabi-ar
ln -sf llvm-ar i686-linux-android-ar
ln -sf llvm-ar x86_64-linux-android-ar
```

**Without these symlinks, Android builds will fail with "ar not found" errors.**

## Features

- **Cross-platform**: macOS, Linux, Windows, iOS, Android
- **Safe API**: Rust-safe wrapper around Crashpad C++ library
- **Flexible Configuration**: Runtime handler configuration
- **Zero-copy**: Minimal overhead for crash reporting
- **Production Ready**: Battle-tested crash reporting solution

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
crashpad = "0.1"
```

## Quick Start

```rust
use crashpad::{CrashpadClient, CrashpadConfig};
use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the Crashpad client
    let client = CrashpadClient::new()?;
    
    // Configure crash reporting
    let config = CrashpadConfig::builder()
        .handler_path("/path/to/crashpad_handler")  // Handler executable
        .database_path("./crashes")                 // Local crash storage
        .url("https://your-crash-server.com/submit") // Upload endpoint
        .build();
    
    // Add application metadata
    let mut annotations = HashMap::new();
    annotations.insert("version".to_string(), env!("CARGO_PKG_VERSION").to_string());
    annotations.insert("build".to_string(), "release".to_string());
    
    // Start crash monitoring
    client.start_with_config(&config, &annotations)?;
    
    // Your application code here
    Ok(())
}
```

## Platform Support

| Platform | Architecture | Status | Handler Type |
|----------|-------------|--------|--------------|
| macOS | x86_64, aarch64 | ✅ Stable | External executable |
| Linux | x86_64, aarch64 | ✅ Stable | External executable |
| Windows | x86_64 | ✅ Stable | External executable |
| iOS | arm64, x86_64 sim | ✅ Stable | In-process |
| Android | arm, arm64, x86, x86_64 | ✅ Stable | External/In-process |

## Handler Integration

The `crashpad_handler` executable must be available at runtime:

### Option 1: Same Directory (Recommended)
Place `crashpad_handler` in the same directory as your application:
```
my-app/
├── my-app
└── crashpad_handler
```

### Option 2: Environment Variable
```bash
export CRASHPAD_HANDLER=/path/to/crashpad_handler
./my-app
```

### Option 3:  Custom Path
```rust
let config = CrashpadConfig::builder()
    .handler_path("/custom/path/crashpad_handler")
    .build();
```

## Building from Source

### Standard Build
```bash
# Clone the repository
git clone https://github.com/bahamoth/crashpad-rs
cd crashpad-rs

# Build (automatically downloads dependencies)
cargo build --release

# Create distribution package
cargo xtask dist
```

### Cross-Compilation

#### Android
```bash
# Install cargo-ndk
cargo install cargo-ndk

# Build for Android (after setting up NDK symlinks)
cargo ndk -t arm64-v8a build --package crashpad
```

#### iOS
```bash
# Add target
rustup target add aarch64-apple-ios

# Build
cargo build --target aarch64-apple-ios
```

See [Cross-Compilation Guide](CONVENTIONS.md#cross-compilation) for detailed instructions.

## Examples

### Basic Example
```bash
cargo run --example crashpad_test_cli
```

### iOS Simulator Test
```bash
cargo build --target aarch64-apple-ios-sim --example ios_simulator_test
```

## Documentation

### For Library Users
- [API Documentation](https://docs.rs/crashpad) - Full API reference
- [Integration Guide](CONVENTIONS.md#integration) - Production integration
- [Platform Notes](CONVENTIONS.md#platform-specific-integration) - Platform-specific considerations

### For Contributors
- [Project Overview](OVERVIEW.md) - Architecture and project structure
- [Development Conventions](CONVENTIONS.md) - Coding standards and workflows
- [Architecture](ARCHITECTURE.md) - Technical design decisions
- [PRD](PRD.md) - Product requirements and roadmap
- [Current Tasks](TASKS.md) - Active development work

## Troubleshooting

### Handler Not Found
- Verify handler executable exists and has execute permissions
- Check `CRASHPAD_HANDLER` environment variable
- Ensure handler architecture matches application

### Android Build Failures
- Verify NDK symlinks are created (see top of README)
- Check `ANDROID_NDK_HOME` is set correctly
- Use `cargo ndk` for simplified builds

### Crashes Not Being Captured
- Confirm handler process is running
- Check database path has write permissions
- Verify network connectivity for uploads

## License

Licensed under either of:
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.

## Contributing

Contributions are welcome! Please read our [Contributing Guide](CONTRIBUTING.md) and [Development Conventions](CONVENTIONS.md).

## Support

- **Issues**: [GitHub Issues](https://github.com/bahamoth/crashpad-rs/issues)
- **Discussions**: [GitHub Discussions](https://github.com/bahamoth/crashpad-rs/discussions)
- **Security**: Report security vulnerabilities to security@example.com
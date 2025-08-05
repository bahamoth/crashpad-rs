# crashpad-rs

Safe Rust bindings for Google Crashpad crash reporting system.

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

- Cross-platform support (macOS, Linux, Windows, iOS, Android)
- Safe Rust API
- Flexible handler configuration
- Build-time handler bundling

## Quick Start

```rust
use crashpad::{CrashpadClient, CrashpadConfig};
use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = CrashpadClient::new()?;
    
    let config = CrashpadConfig::builder()
        .handler_path("/path/to/crashpad_handler")
        .database_path("./crashes")
        .url("https://your-crash-server.com/submit")
        .build();
    
    let mut annotations = HashMap::new();
    annotations.insert("version".to_string(), env!("CARGO_PKG_VERSION").to_string());
    
    client.start_with_config(&config, &annotations)?;
    
    // Your application code here
    Ok(())
}
```

## Running Examples

```bash
# Interactive CLI for testing Crashpad functionality
cargo run --example crashpad_test_cli

```

## Development Tasks

This project uses `cargo xtask` for development tasks:

```bash
# Build the project
cargo xtask build

# Build in release mode
cargo xtask build --release

# Create distribution package
cargo xtask dist

# Run tests
cargo xtask test

# Clean build artifacts
cargo xtask clean
```

## Distribution

When distributing your application:

1. Build your application in release mode
2. Run `cargo xtask dist` to create a distribution package
3. Copy `crashpad_handler` from the dist folder to your application directory

The handler executable must be placed in one of these locations:
- Same directory as your application (recommended)
- System PATH (`/usr/local/bin`, `/usr/bin` on Unix)
- Location specified by `CRASHPAD_HANDLER` environment variable

## Platform Support

- ✅ macOS (x86_64, aarch64)
- ✅ Linux (x86_64, aarch64)
- ✅ Windows (x86_64)
- ✅ iOS (arm64, x86_64 simulator)
- ✅ Android (arm, arm64, x86, x86_64)

## Cross-Compilation

### Android

The easiest way to build for Android is using cargo-ndk:

```bash
# Install cargo-ndk
cargo install cargo-ndk

# IMPORTANT: Create symlinks first (see "Android Build Requirements" section above)

# Build for Android
cargo ndk -t arm64-v8a build --package crashpad-sys
```

**Note**: The symlink creation is mandatory for NDK r22+. The build will fail without it.

For more detailed instructions and other platforms, see [CROSS_COMPILE.md](CROSS_COMPILE.md).

## Environment Variables

This crate uses the following environment variables:

- `CRASHPAD_HANDLER` - Path to the crashpad_handler executable. If not set, the library will search for it in:
  1. Same directory as your executable (for deployed apps)
  2. `third_party/crashpad_checkout/crashpad/out/{platform}/crashpad_handler` (for development)
  3. System paths (`/usr/local/bin`, `/usr/bin` on Unix)

**Note**: For development, you can set `CRASHPAD_HANDLER` environment variable or use `.cargo/config.toml` to configure the path. For production deployments, place the handler executable in the same directory as your application.

## License

Licensed under either of:
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.
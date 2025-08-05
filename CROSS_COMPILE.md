# Cross-Compilation Guide

This guide explains how to build crashpad-rs for different target platforms.

## Android

### Quick Start (Recommended)

Using cargo-ndk is the easiest way to build for Android:

```bash
# Install cargo-ndk
cargo install cargo-ndk

# Build for specific architecture
cargo ndk -t arm64-v8a build --package crashpad-sys    # ARM64 (most devices)
cargo ndk -t armeabi-v7a build --package crashpad-sys  # ARMv7 (older devices)
cargo ndk -t x86_64 build --package crashpad-sys       # x86_64 (emulators)
cargo ndk -t x86 build --package crashpad-sys          # x86 (older emulators)
```

### Prerequisites

1. Install Android NDK:
   - Download from [Android NDK Downloads](https://developer.android.com/ndk/downloads)
   - Or use Android Studio's SDK Manager
   - Set `ANDROID_NDK_HOME` environment variable:
     ```bash
     export ANDROID_NDK_HOME=/path/to/android-ndk
     ```

2. Install Rust targets:
   ```bash
   rustup target add aarch64-linux-android
   rustup target add armv7-linux-androideabi
   rustup target add x86_64-linux-android
   rustup target add i686-linux-android
   ```

### Manual Build (Alternative)

If you prefer not to use cargo-ndk:

```bash
# Add NDK tools to PATH
export PATH=$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin:$PATH

# Build
cargo build --package crashpad-sys --target aarch64-linux-android
```

## iOS

iOS builds require macOS with Xcode installed.

```bash
# Add iOS targets
rustup target add aarch64-apple-ios
rustup target add aarch64-apple-ios-sim

# Build for iOS device
cargo build --target aarch64-apple-ios

# Build for iOS simulator
cargo build --target aarch64-apple-ios-sim
```

## Windows (from Linux/WSL)

### Setup

Install MinGW-w64:
```bash
# Ubuntu/Debian
sudo apt install mingw-w64

# Fedora
sudo dnf install mingw64-gcc
```

### Building

```bash
# Add Windows target
rustup target add x86_64-pc-windows-gnu

# Build
cargo build --target x86_64-pc-windows-gnu
```

## Troubleshooting

### Android Build Errors

1. **"error adding symbols: file in wrong format"**
   - Make sure you're not trying to build host-only tools (like xtask) for Android
   - Build only the library: `cargo build --package crashpad-sys --target aarch64-linux-android`

2. **"llvm-ar: not found"**
   - Ensure `$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin` is in your PATH

3. **"ANDROID_NDK_HOME not set"**
   - Set the environment variable: `export ANDROID_NDK_HOME=/path/to/ndk`

### General Tips

- The `.cargo/config.toml` file in this project sets up the correct `ar` tool for Android targets
- For CI/CD, make sure to install and configure the appropriate toolchains
- When in doubt, use cargo-ndk for Android builds as it handles most configuration automatically
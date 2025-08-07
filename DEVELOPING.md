# Development Guide

This guide covers the development workflow for crashpad-rs, including environment setup, building, testing, and contributing.

## Table of Contents

- [Prerequisites](#prerequisites)
- [Development Environment Setup](#development-environment-setup)
- [Building](#building)
- [Testing](#testing)
- [Cross-Compilation](#cross-compilation)
- [Code Quality](#code-quality)
- [Debugging](#debugging)
- [Troubleshooting](#troubleshooting)

## Prerequisites

### Required Tools

- **Rust**: 1.70+ (install via [rustup](https://rustup.rs/))
- **Python**: 3.8+ (required for depot_tools)
- **Git**: For source control and depot_tools
- **C++ Compiler**: 
  - macOS: Xcode Command Line Tools (`xcode-select --install`)
  - Linux: GCC or Clang (`apt install build-essential`)
  - Windows: Visual Studio 2019+ or MinGW

### Platform-Specific Requirements

#### macOS/iOS
- Xcode (for iOS development)
- iOS Simulator (comes with Xcode)

#### Android
- Android NDK (r23 or later, despite Crashpad docs saying r21)

> ⚠️ **IMPORTANT: Android NDK Symlink Workaround** ⚠️
> 
> **This is a temporary ad-hoc solution!** The issue is in Crashpad's mini_chromium build configuration
> ([mini_chromium/build/config/BUILD.gn#L481-L497](https://github.com/chromium/mini_chromium/blob/main/build/config/BUILD.gn#L481-L497)).   
> It constructs NDK tool paths incorrectly by concatenating `tool_prefix + android_api_level + "-clang"` (e.g. `aarch64-linux-android21-clang`),  
> but then looks for `tool_prefix + "-clang"` without the API level. 
> Until this is fixed upstream, you MUST create these symlinks manually:  
> 
> ```bash
> # ⚠️ REQUIRED: Create symlinks for NDK tools (temporary workaround)
> cd $ANDROID_NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin
> 
> # For ARM64
> ln -sf aarch64-linux-android23-clang aarch64-linux-android-clang
> ln -sf aarch64-linux-android23-clang++ aarch64-linux-android-clang++
> 
> # For x86_64
> ln -sf x86_64-linux-android23-clang x86_64-linux-android-clang
> ln -sf x86_64-linux-android23-clang++ x86_64-linux-android-clang++
> ```
> 
> **Without these symlinks, Android builds WILL FAIL with "compiler not found" errors!**
- Set environment variables:
  ```bash
  export ANDROID_NDK_HOME=/path/to/android-ndk
  export ANDROID_NDK_ROOT=$ANDROID_NDK_HOME
  ```

#### Linux
- Additional packages:
  ```bash
  sudo apt install ninja-build pkg-config libssl-dev
  ```

## Development Environment Setup

### 1. Clone the Repository

```bash
git clone https://github.com/your-org/crashpad-rs.git
cd crashpad-rs
```

### 2. Install Development Tools

We provide an xtask command to install all required development tools:

```bash
# Install cargo-nextest and cargo-ndk
cargo xtask install-tools
```

This installs:
- **cargo-nextest**: Test runner with process isolation (required for Crashpad's global state)
- **cargo-ndk**: Android cross-compilation helper

### 3. Verify Setup

```bash
# Check build environment
cargo build --package crashpad-sys

# Run tests
cargo nextest run
```

## Building

### Native Build

```bash
# Build the FFI layer
cargo build --package crashpad-sys

# Build the safe wrapper
cargo build --package crashpad

# Build everything in release mode
cargo build --release
```

### Clean Build

```bash
# Clean all build artifacts including native dependencies
make clean

# Or manually:
rm -rf target/ third_party/
cargo clean
```

### What's happening in the Build?

The build system automatically:
1. Downloads crashpad and depot_tools (Chromium build tools)
2. Sync mini-chromium via gclient
3. Configures build with GN
4. Compiles with Ninja
5. Generates Rust bindings with bindgen
6. Creates static libraries

Build artifacts are cached in `third_party/` (gitignored).

## Cross-Compilation

### iOS (Device)

```bash
# Add target
rustup target add aarch64-apple-ios

# Build
cargo build --target aarch64-apple-ios --package crashpad
```

### iOS (Simulator)

```bash
# ARM64 (M1/M2 Macs)
rustup target add aarch64-apple-ios-sim
cargo build --target aarch64-apple-ios-sim --package crashpad

# x86_64 (Intel Macs)
rustup target add x86_64-apple-ios-sim
cargo build --target x86_64-apple-ios-sim --package crashpad
```

### Android

```bash
# Install cargo-ndk
cargo install cargo-ndk

# Add targets
rustup target add aarch64-linux-android
rustup target add armv7-linux-androideabi
rustup target add x86_64-linux-android

# Build for specific architectures
cargo ndk -t arm64-v8a build --package crashpad-sys
cargo ndk -t armeabi-v7a build --package crashpad-sys
cargo ndk -t x86_64 build --package crashpad-sys
```

### Linux (from macOS)

```bash
# Using cross
cargo install cross
cross build --target x86_64-unknown-linux-gnu
```

### Windows

**Note: Windows builds are currently untested and may not work.**

## Testing

We use cargo-nextest for all tests because Crashpad's out-of-process handler requires isolated process execution:

```bash
# Run all tests
cargo nextest run

# Run specific package tests
cargo nextest run -p crashpad

# Run with verbose output
cargo nextest run --verbose

# Run ignored tests
cargo nextest run --run-ignored all

# macOS-specific tests
cargo nextest run -p crashpad --test macos_test
```

### Platform-Specific Tests

#### iOS Simulator

```bash
# 1. Install and set up iOS Simulator
# Requires Xcode installed from App Store or developer.apple.com

# 2. List available simulators
xcrun simctl list devices

# 3. Create a new simulator if needed (example: iPhone 15 Pro)
xcrun simctl create "iPhone 15 Pro" "iPhone 15 Pro" iOS17.5

# 4. Boot the simulator
xcrun simctl boot "iPhone 15 Pro"
# Or boot by device ID
xcrun simctl boot {DEVICE_ID}

# 5. Build for iOS simulator (ARM64 for M1/M2 Macs)
cargo build --target aarch64-apple-ios-sim --example ios_simulator_test

# 6. Run the test in simulator (FIRST RUN - generates crash)
xcrun simctl spawn booted target/aarch64-apple-ios-sim/debug/examples/ios_simulator_test
# This will crash and create intermediate dump

# 7. Run again to process intermediate dumps (SECOND RUN - converts to minidump)
xcrun simctl spawn booted target/aarch64-apple-ios-sim/debug/examples/ios_simulator_test
# iOS in-process handler requires app restart to process crashes from previous session

# 8. Find crash dumps (get device ID first)
xcrun simctl list devices | grep Booted
# Example output: iPhone 15 Pro (DEVICE_ID) (Booted)

# 9. Check for crash dumps in default locations (relative to simulator's data directory):
# Default database path: ./crashpad_database (as configured in ios_simulator_test.rs)
# Intermediate dumps (created on crash):
ls -la ~/Library/Developer/CoreSimulator/Devices/{DEVICE_ID}/data/crashpad_database/pending-serialized-ios-dump/
# Processed minidumps (created on second run):
ls -la ~/Library/Developer/CoreSimulator/Devices/{DEVICE_ID}/data/crashpad_database/pending/
ls -la ~/Library/Developer/CoreSimulator/Devices/{DEVICE_ID}/data/crashpad_database/completed/

# 10. Clean up simulator data if needed
xcrun simctl erase {DEVICE_ID}

# 11. Shutdown simulator when done
xcrun simctl shutdown booted
```

#### Android

```bash
# 1. Install Android SDK and emulator
# Via Android Studio or command line tools from developer.android.com

# 2. Add emulator and platform-tools to PATH (if not already done)
export PATH=$PATH:$ANDROID_HOME/emulator:$ANDROID_HOME/platform-tools

# 3. List available AVDs (Android Virtual Devices)
emulator -list-avds

# 4. Download system images first (if not already installed)
sdkmanager "system-images;android-33;google_apis;arm64-v8a"

# 5. Create AVD (example: Pixel 7 with API 33)
avdmanager create avd -n Pixel_7_API_33 -k "system-images;android-33;google_apis;arm64-v8a" --device "pixel_7"

# 6. Start emulator (headless for CI/testing)
emulator -avd Pixel_7_API_33 -no-window -no-audio -no-boot-anim

# 7. Wait for emulator to boot completely
adb wait-for-device
adb shell getprop sys.boot_completed  # Should return "1" when ready

# 8. Build with cargo-ndk
cargo ndk -t arm64-v8a build --package crashpad --example crashpad_test_cli

# 9. Push executable AND handler to emulator/device
# Note: Handler is renamed to .so extension for APK distribution (not actually a shared library)
adb push target/aarch64-linux-android/debug/examples/crashpad_test_cli /data/local/tmp/
adb push target/aarch64-linux-android/debug/libcrashpad_handler.so /data/local/tmp/

# 10. Make executable and run
adb shell chmod +x /data/local/tmp/crashpad_test_cli
adb shell chmod +x /data/local/tmp/libcrashpad_handler.so
adb shell /data/local/tmp/crashpad_test_cli

# 11. Check crash dumps in default locations:
# Default database path: ./crashpad_database (relative to /data/local/tmp/)
adb shell ls -la /data/local/tmp/crashpad_database/pending/
adb shell ls -la /data/local/tmp/crashpad_database/completed/

# 12. Pull crash dumps to local machine for analysis
adb pull /data/local/tmp/crashpad_database/pending/ ./android_crashes/
adb pull /data/local/tmp/crashpad_database/completed/ ./android_crashes/

# 13. Clean up test files
adb shell rm -rf /data/local/tmp/crashpad_test_cli
adb shell rm -rf /data/local/tmp/libcrashpad_handler.so
adb shell rm -rf /data/local/tmp/crashpad_database/

# 14. Stop emulator when done
adb emu kill
# Or press Ctrl+C in emulator terminal
```

### Example Programs

```bash
# Basic crash test CLI
cargo run --example crashpad_test_cli

# iOS simulator test (requires iOS simulator)
cargo build --target aarch64-apple-ios-sim --example ios_simulator_test
```

## Code Quality

### Formatting

```bash
# Format all code
cargo fmt --all

# Check formatting without changes
cargo fmt --all -- --check
```

### Linting

```bash
# Run clippy with all targets and features
cargo clippy --all-targets --all-features -- -D warnings

# Fix clippy suggestions automatically
cargo clippy --fix --all-targets --all-features
```

### Pre-Commit Checklist

Before committing, always run:

```bash
# 1. Format code
cargo fmt --all

# 2. Run clippy
cargo clippy --all-targets --all-features -- -D warnings

# 3. Run tests
cargo nextest run

# 4. Build all packages
cargo build --all
```

## Debugging

### Verbose Build Output

```bash
# Enable verbose build output
CRASHPAD_VERBOSE=1 cargo build --package crashpad-sys

# See actual compiler commands
cargo build -vv
```

### Examining Crash Dumps

#### macOS/Linux

```bash
# Check for crash dumps
ls -la ./crashpad_database/pending/
ls -la ./crashpad_database/completed/

# Use minidump_stackwalk (install separately)
minidump_stackwalk crash.dmp symbols/
```

#### iOS

```bash
# Find simulator device ID
xcrun simctl list devices

# Check iOS crash dumps
ls -la ~/Library/Developer/CoreSimulator/Devices/{DEVICE_ID}/data/crashpad_database/

# Intermediate dumps (iOS specific)
ls -la ~/Library/Developer/CoreSimulator/Devices/{DEVICE_ID}/data/crashpad_database/pending-serialized-ios-dump/
```

### Debug Logging

Set environment variables for debug output:

```bash
# Enable verbose output
export CRASHPAD_VERBOSE=1

# Rust backtrace
export RUST_BACKTRACE=1
```

## Troubleshooting

### Common Build Issues

#### Android NDK Not Found

**Problem**: Build fails with NDK not found
```
Android target but NDK not found
```

**Solution**: Set NDK environment variables:
```bash
export ANDROID_NDK_HOME=/path/to/ndk
export ANDROID_NDK_ROOT=$ANDROID_NDK_HOME
```

#### depot_tools Download Failures

**Problem**: Failed to clone depot_tools or build errors after depot_tools update

**Solution**: 
1. Check if Crashpad's build requirements have changed upstream
2. Clean everything and rebuild: `make clean && cargo build`
3. May need to update `build.rs` if Crashpad's build process has changed

### Test Failures

#### Global State Conflicts

**Problem**: Tests fail when run with `cargo test`

**Solution**: Use `cargo nextest run` for process isolation

#### iOS Simulator Tests

**Problem**: No crash dumps generated

**Solution**: 
1. Ensure simulator is running
2. Check correct device ID
3. Process intermediate dumps require app restart after crash

### Platform-Specific Issues


#### iOS In-Process Handler

iOS uses an in-process handler (no separate executable). Crashes are captured as intermediate dumps and converted to minidumps on next app launch.

## Project Structure

```
crashpad-rs/
├── crashpad-sys/          # Low-level FFI bindings
│   ├── build.rs          # Build script orchestration
│   ├── build/            # Build system modules
│   │   ├── config.rs     # Platform configuration
│   │   └── phases.rs     # Build phases
│   ├── wrapper.h         # C API declarations
│   └── crashpad_wrapper.cc # C++ bridge implementation
├── crashpad/             # Safe Rust wrapper
│   ├── src/
│   │   ├── client.rs    # CrashpadClient implementation
│   │   ├── config.rs    # Configuration builder
│   │   └── lib.rs       # Public API
│   ├── examples/         # Example programs
│   └── tests/           # Integration tests
├── xtask/               # Development automation
└── third_party/         # Build dependencies (gitignored)
    ├── depot_tools/     # Chromium build tools
    └── crashpad_checkout/ # Crashpad source
```

## Contributing

1. **Read the docs**: Start with ARCHITECTURE.md and CONVENTIONS.md
2. **Check TASKS.md**: See available work items
3. **Test thoroughly**: Use nextest for all testing
4. **Follow conventions**: Run formatter and linter before commit
5. **Update documentation**: Keep docs in sync with code changes

### Commit Message Format

Follow [Conventional Commits](https://www.conventionalcommits.org/):
```
feat(ios): add iOS simulator support
fix(build): resolve MIG linking issue on macOS
docs: update cross-compilation guide
```

## Additional Resources

- [README.md](README.md) - User documentation
- [ARCHITECTURE.md](ARCHITECTURE.md) - System design and FFI patterns
- [CONVENTIONS.md](CONVENTIONS.md) - Coding standards
- [CLAUDE.md](CLAUDE.md) - AI assistant guidelines
- [Google Crashpad Documentation](https://chromium.googlesource.com/crashpad/crashpad/+/master/README.md)
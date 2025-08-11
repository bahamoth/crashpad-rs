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
- **Git**: For source control and submodules
- **C++ Compiler**: 
  - macOS: Xcode Command Line Tools (`xcode-select --install`)
  - Linux: GCC or Clang (`apt install build-essential`)
  - Windows: Visual Studio 2019+ or MinGW

> **Note**: Python and depot_tools are no longer required! Git submodules handle all dependencies.

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
> It looks for archiver tools with incorrect names (e.g. `aarch64-linux-android-ar`).  
> Until this is fixed upstream, you MUST create these symlinks manually:  
> 
> **For NDK r27+ (recommended):**
> ```bash
> # ⚠️ REQUIRED: Create symlinks for NDK archiver (temporary workaround)
> cd $ANDROID_NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin
> 
> # Only ar needs symlinks in r27+
> ln -sf llvm-ar aarch64-linux-android-ar
> ln -sf llvm-ar arm-linux-androideabi-ar
> ln -sf llvm-ar x86_64-linux-android-ar
> ln -sf llvm-ar i686-linux-android-ar
> ```
> 
> **For NDK r26 and earlier:**
> ```bash
> # ⚠️ REQUIRED: Create symlinks for NDK tools (temporary workaround)
> cd $ANDROID_NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin
> 
> # For ARM64
> ln -sf aarch64-linux-android21-clang aarch64-linux-android-clang
> ln -sf aarch64-linux-android21-clang++ aarch64-linux-android-clang++
> ln -sf llvm-ar aarch64-linux-android-ar
> 
> # For x86_64
> ln -sf x86_64-linux-android21-clang x86_64-linux-android-clang
> ln -sf x86_64-linux-android21-clang++ x86_64-linux-android-clang++
> ln -sf llvm-ar x86_64-linux-android-ar
> ```
> 
> **Without these symlinks, Android builds WILL FAIL with "ar not found" or "compiler not found" errors!**
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

# Initialize all submodules (required!)
git submodule update --init --recursive
```

### 2. Submodule Structure

This project uses Git submodules to manage Crashpad and its dependencies following the exact DEPS hierarchy.

```
crashpad-sys/third_party/
├── crashpad/           # Google Crashpad (submodule)
├── mini_chromium/      # Base library (submodule)
├── googletest/         # Test framework (submodule)
├── zlib/              # Compression library (submodule)
├── libfuzzer/         # Fuzzing library (submodule)
├── edo/               # iOS library (submodule)
└── lss/               # Linux syscalls (submodule)
```

The build system automatically creates symlinks/junctions inside `crashpad/third_party/` to these dependencies.

### 3. Install Development Tools

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
# Build the FFI layer (note: package name is crashpad-rs-sys)
cargo build --package crashpad-rs-sys

# Build the safe wrapper
cargo build --package crashpad-rs

# Build everything in release mode
cargo build --release

# Create distribution package with handler
cargo xtask dist
```

### Clean Build

```bash
# Clean all build artifacts including native dependencies
make clean

# Or manually:
rm -rf target/ crashpad-sys/third_party/crashpad/out/
cargo clean
```

### What's happening in the Build?

The build system automatically:
1. Downloads GN and Ninja binaries directly from CIPD
2. Creates symlinks/junctions for submodule dependencies
3. Configures build with GN
4. Compiles with Ninja
5. Generates Rust bindings with bindgen
6. Creates static libraries

Build tools are cached in OS-specific cache directories.

## Native Dependencies Version Management

crashpad-rs uses Git submodules to pin specific versions of native dependencies:

### Current Submodule Versions

All dependency versions are managed through Git submodules in `crashpad-sys/third_party/`:
- crashpad
- mini_chromium  
- googletest
- zlib
- libfuzzer
- edo
- lss (for Android/Linux)

### Updating Submodule Versions

To update a specific dependency:

```bash
# Update specific submodule to latest
cd crashpad-sys/third_party/crashpad
git checkout <new_commit>
cd ../../..
git add crashpad-sys/third_party/crashpad
git commit -m "chore: update crashpad to <new_commit>"

# Or update all submodules to latest
git submodule update --remote --merge
```

### Build Tool Versions

GN and Ninja versions are defined in `crashpad-sys/build/tools.rs`:
- GN version is matched to Crashpad's requirements
- Ninja is downloaded from GitHub releases

These tools are cached in OS-specific cache directories:
- macOS: `~/Library/Caches/crashpad-rs/`
- Linux: `~/.cache/crashpad-rs/`
- Windows: `%LOCALAPPDATA%\crashpad-rs\Cache\`

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
cargo ndk -t arm64-v8a build --package crashpad-rs-sys
cargo ndk -t armeabi-v7a build --package crashpad-rs-sys
cargo ndk -t x86_64 build --package crashpad-rs-sys
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

### Quick Test Commands

```bash
# Run only unit tests (fast, no handler needed)
cargo test --lib

# Run only integration tests with process isolation (needs handler)
cargo nextest run --test '*'

# Run all tests with nextest
cargo nextest run
```

### Detailed Testing

We use cargo-nextest for integration tests because Crashpad's out-of-process handler requires isolated process execution:

```bash
# Unit tests only (in src/ files)
cargo test --lib
cargo test --lib -p crashpad-rs  # Specific package

# Integration tests only (in tests/ directory)
cargo nextest run --test '*'
cargo nextest run --test integration_test  # Specific test file
cargo nextest run --test macos_test  # Platform-specific tests

# All tests
cargo nextest run
cargo nextest run -p crashpad-rs  # Specific package

# With verbose output
cargo nextest run --verbose

# Run ignored tests
cargo nextest run --run-ignored all
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
cargo ndk -t arm64-v8a build --package crashpad-rs --example crashpad_test_cli

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
CRASHPAD_VERBOSE=1 cargo build --package crashpad-rs-sys

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

#### Submodule Issues

**Problem**: Missing dependencies or build failures

**Solution**: 
1. Initialize submodules: `git submodule update --init --recursive`
2. Clean everything and rebuild: `make clean && cargo build`
3. Check that symlinks/junctions were created in `crashpad-sys/third_party/crashpad/third_party/`

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
├── crashpad-sys/          # Low-level FFI bindings (publishes as crashpad-rs-sys)
│   ├── build.rs          # Build script orchestration
│   ├── build/            # Build system modules
│   │   ├── config.rs     # Platform configuration
│   │   └── phases.rs     # Build phases
│   ├── wrapper.h         # C API declarations
│   ├── crashpad_wrapper.cc # C++ bridge implementation
│   └── third_party/      # Git submodules
│       ├── crashpad/     # Crashpad source
│       ├── mini_chromium/ # Base library
│       ├── googletest/   # Test framework
│       ├── zlib/         # Compression
│       ├── libfuzzer/    # Fuzzing
│       ├── edo/          # iOS library
│       └── lss/          # Linux syscalls
├── crashpad/             # Safe Rust wrapper (publishes as crashpad)
│   ├── src/
│   │   ├── client.rs    # CrashpadClient implementation
│   │   ├── config.rs    # Configuration builder
│   │   └── lib.rs       # Public API
│   ├── examples/         # Example programs
│   └── tests/           # Integration tests
└── xtask/               # Development automation
```

## Packaging for crates.io

### Package Structure

The project uses different names for directories and crates.io packages:

| Directory | Package Name | Reason |
|-----------|-------------|--------|
| `crashpad-sys/` | `crashpad-rs-sys` | Avoids conflict with existing `crashpad-sys` crate |
| `crashpad/` | `crashpad-rs` | Consistent naming with FFI package |

### Publishing Process

1. **Prepare symlinks** (required for packaging):
   ```bash
   # Create symlinks for Crashpad dependencies
   cargo xtask symlink
   ```

2. **Package the FFI bindings**:
   ```bash
   # Package crashpad-rs-sys (from crashpad-sys directory)
   cargo package -p crashpad-rs-sys
   
   # Verify the package (optional)
   cargo package -p crashpad-rs-sys --list
   ```

3. **Package the safe wrapper**:
   ```bash
   # Package crashpad-rs
   cargo package -p crashpad-rs
   ```

4. **Publish to crates.io** (maintainers only):
   ```bash
   # Publish in dependency order
   cargo publish -p crashpad-rs-sys
   cargo publish -p crashpad-rs
   ```

### How Packaging Works

The build system handles cargo package specially:

1. **Symlink Creation**: Run `cargo xtask symlink` to pre-create dependency symlinks before packaging
2. **Package Detection**: `build.rs` detects when running in `cargo package` environment
3. **Build Skipping**: During packaging, the actual Crashpad build is skipped
4. **Symlink Following**: cargo follows symlinks and includes actual files in the package

This approach ensures the package includes all necessary source files without requiring build-time symlink creation.

### Troubleshooting Package Issues

#### "Source directory was modified" Error
**Problem**: cargo package fails with verification error
**Solution**: Run `cargo xtask symlink` before packaging

#### Package Size Concerns
The packaged crate includes Crashpad source code and dependencies (~3-4MB compressed).
This is necessary since Crashpad must be built from source on the target system.

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
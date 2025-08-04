# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Rust wrapper for Google Crashpad crash reporting library with cross-platform support.

### Architecture
- `crashpad-sys`: Low-level FFI bindings using bindgen
- `crashpad`: Safe Rust API wrapper
- `third_party/`: Dependencies managed by build.rs (not git submodules)

## Build Requirements

1. **Build dependencies**
   - C/C++ compiler (gcc/clang)
   - bindgen dependencies (libclang)
   - git (for cloning dependencies)
   - Python (for depot_tools)
   - Platform-specific SDKs (Android NDK for Android builds)

Note: depot_tools is automatically downloaded to `third_party/depot_tools` during the build process.

## Common Commands

```bash
# Build the project (will auto-clone dependencies)
cargo build --package crashpad-sys
cargo build --package crashpad

# Clean build
rm -rf third_party && cargo clean
cargo build
```

## Development History

### Phase 1: Initial Setup with Submodules (Failed)
- Tried using Crashpad as git submodule
- Issue: Crashpad expects gclient to manage dependencies
- mini_chromium and other deps were not properly synced

### Phase 2: build.rs Managed Dependencies (Current)
- Removed submodules approach
- build.rs now manages all dependencies:
  - depot_tools (automatically cloned to third_party/)
  - Crashpad (using proper gclient workflow)
  - All Crashpad dependencies via gclient sync
- Uses proper Chromium build workflow:
  - Creates .gclient configuration
  - Uses gclient sync for dependency management
  - Generates build files with gn
  - Builds with ninja
- Added `/third_party/` to .gitignore

## Current Status

- Proper Chromium-style build system implemented
- depot_tools automatically managed in third_party/
- Crashpad built using official gclient/gn/ninja workflow
- Cross-platform support for macOS, iOS, Linux, Android, Windows

## Platform Support

- macOS (x64)
- iOS (arm64, x64)
- Linux (x64)
- Android (arm, arm64, x86, x64)
- Windows (x64)

## Development Notes

- Using standard Rust FFI pattern: `-sys` crate for raw bindings, main crate for safe wrapper
- build.rs handles:
  - Cloning all dependencies to correct locations
  - Cross-platform Crashpad compilation using gn/ninja
- bindgen generates FFI bindings from wrapper.h
- CargoCallbacks deprecation warning can be fixed by using `CargoCallbacks::new()`
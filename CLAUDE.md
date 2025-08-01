# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Rust wrapper for Google Crashpad crash reporting library with cross-platform support.

### Architecture
- `crashpad-sys`: Low-level FFI bindings using bindgen
- `crashpad`: Safe Rust API wrapper
- `third_party/`: Dependencies managed by build.rs (not git submodules)

## Build Requirements

1. **depot_tools** (required for gn and ninja)
   ```bash
   export PATH=$HOME/projects/depot_tools:$PATH
   ```

2. **Build dependencies**
   - C/C++ compiler (gcc/clang)
   - bindgen dependencies
   - git (for cloning dependencies)
   - Platform-specific SDKs (Android NDK for Android builds)

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
- build.rs now clones and manages all dependencies:
  - Crashpad
  - mini_chromium (to correct nested path)
  - googletest
  - linux-syscall-support
- Added `/third_party/` to .gitignore

## Current Issues (WIP)

1. **gn checkout detection**: 
   - Error: "Could not find checkout in any parent of the current path"
   - gn expects depot_tools style checkout
   - May need buildtools or .gclient setup

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
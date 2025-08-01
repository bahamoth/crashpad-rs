# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Rust wrapper for Google Crashpad crash reporting library with cross-platform support.

### Architecture
- `crashpad-sys`: Low-level FFI bindings using bindgen
- `crashpad`: Safe Rust API wrapper
- `third_party/crashpad`: Google Crashpad as git submodule

## Build Requirements

1. **depot_tools** (required for gn and ninja)
   ```bash
   export PATH=$HOME/projects/depot_tools:$PATH
   ```

2. **Build dependencies**
   - C/C++ compiler (gcc/clang)
   - bindgen dependencies
   - Platform-specific SDKs (Android NDK for Android builds)

## Common Commands

```bash
# Build the project
cargo build --package crashpad-sys
cargo build --package crashpad

# Sync Crashpad dependencies
cd third_party/crashpad
gclient sync
```

## Current Issues (WIP)

1. **mini_chromium dependency**: gclient sync not properly fetching mini_chromium subdirectory
   - Expected: `third_party/mini_chromium/mini_chromium/`
   - Actual: Only `third_party/mini_chromium/` exists
   - Need to fix gclient configuration

## Platform Support

- macOS (x64)
- iOS (arm64, x64)
- Linux (x64)
- Android (arm, arm64, x86, x64)
- Windows (x64)

## Development Notes

- Using standard Rust FFI pattern: `-sys` crate for raw bindings, main crate for safe wrapper
- build.rs handles cross-platform Crashpad compilation using gn/ninja
- bindgen generates FFI bindings from wrapper.h
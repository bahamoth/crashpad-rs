# Pre-built Package Structure

## Required Files for crates.io Distribution

Based on the build analysis, here are the essential files needed for a minimal pre-built package:

### 1. Static Libraries (.a files)

#### Core Libraries (Required)
Located in `obj/` subdirectories:
- `client/libclient.a` - Crashpad client library
- `util/libutil.a` - Utility functions
- `minidump/libformat.a` - Minidump format support
- `minidump/libminidump.a` - Minidump handling
- `snapshot/libsnapshot.a` - Process snapshot
- `snapshot/libcontext.a` - Context capture
- `handler/libcommon.a` - Handler common code
- `third_party/mini_chromium/mini_chromium/base/libbase.a` - Chromium base library

#### Platform-specific Libraries
**macOS/iOS only:**
- `handler/libmig_output.a` - MIG generated code

**iOS only (additional):**
- `util/libnet.a` - Network utilities

### 2. Headers (for wrapper compilation)

Required for Phase 4 (wrapper compilation):
```
include/
├── client/
│   └── *.h files
├── util/
│   └── *.h files  
├── handler/
│   └── *.h files
├── minidump/
│   └── *.h files
├── snapshot/
│   └── *.h files
└── third_party/
    └── mini_chromium/
        └── mini_chromium/
            └── base/
                └── *.h files
```

### 3. Handler Binary

**Desktop platforms (macOS, Linux, Windows):**
- `crashpad_handler` (or `.exe` on Windows)

**Android:**
- `libcrashpad_handler.so` (renamed for APK packaging)

**iOS:**
- No handler (in-process handler)

## Package Size Optimization

### Files NOT needed:
- Test libraries (`libtest.a`, `libgoogletest*.a`, `libgooglemock*.a`)
- Test support libraries (`libtest_support.a`)
- Compatibility layer (`compat/libcompat.a`)
- Build intermediates (`.o` files)
- Debug symbols (unless specifically requested)

### Estimated Package Sizes

Based on current builds:
- **Minimal package**: ~30-40 MB per platform
  - Static libraries: ~25 MB
  - Headers: ~2 MB
  - Handler: ~5-22 MB (platform dependent)

- **Full build output**: ~200+ MB per platform
  - Includes all test libraries and intermediates

## Prebuilt Bundle Structure

The build system automatically creates a minimal prebuilt bundle at:
`target/{target}/{profile}/prebuilt_bundle/`

```
prebuilt_bundle/
├── lib/
│   ├── libclient.a
│   ├── libutil.a
│   ├── libformat.a
│   ├── libminidump.a
│   ├── libsnapshot.a
│   ├── libcontext.a
│   ├── libcommon.a
│   ├── libbase.a
│   ├── libmig_output.a     # macOS/iOS only
│   └── libnet.a            # iOS only
├── include/
│   ├── client/
│   ├── util/
│   ├── handler/
│   ├── minidump/
│   ├── snapshot/
│   └── third_party/
│       └── mini_chromium/
├── bin/
│   └── crashpad_handler    # Not present on iOS
│       # or libcrashpad_handler.so (Android)
│       # or crashpad_handler.exe (Windows)
└── metadata.json
    {
      "crashpad_commit": "sha",
      "depot_tools_commit": "sha",
      "target": "target-triple",
      "profile": "debug/release",
      "build_date": "ISO-8601",
      "bundle_version": "1.0"
    }
```

## Bundle Creation

The prebuilt bundle is created automatically during Phase 3 (build) by calling `create_prebuilt_bundle()`. It:

1. **Selects essential libraries** - Excludes test and mock libraries
2. **Copies required headers** - Only headers needed for compilation
3. **Includes handler binary** - Platform-specific naming
4. **Generates metadata** - Version and build information
5. **Reports size reduction** - Typically 91-93% smaller than full build

### Size Comparison

| Build Type | Platform | Full Build | Prebuilt Bundle | Reduction |
|------------|----------|------------|-----------------|-----------|
| Debug | macOS | 679MB | 59MB | 91% |
| Debug | iOS | ~650MB | 55MB | 91% |
| Debug | Android | ~700MB | 86MB | 88% |
| Release | macOS | 88MB | 5.8MB | 93% |

## Building with xtask

Create a prebuilt bundle using xtask:

```bash
# Default target (host platform)
cargo xtask prebuild

# Specific target
cargo xtask prebuild --target aarch64-apple-ios

# Custom output directory (optional)
cargo xtask prebuild --output /path/to/output
```

This runs phases 1-3 and creates both:
- `target/{target}/{profile}/crashpad_build/` - Full build output
- `target/{target}/{profile}/prebuilt_bundle/` - Minimal distribution bundle

## Usage in build.rs

The emit_link phase automatically detects prebuilt bundles:
1. Checks if `crashpad_build/lib/` exists (prebuilt bundle structure)
2. If yes: Links from `lib/` directory
3. If no: Links from `obj/` subdirectories (full build structure)

Future enhancements (planned):
- Environment variable `CRASHPAD_PREBUILT_DIR` support
- GitHub artifacts integration
- Automatic download of prebuilt bundles

## Platform-specific Notes

### macOS
- Requires `libtool` for static library creation
- Links against system frameworks
- Needs `bsm` dylib

### iOS
- In-process handler (no separate binary)
- Additional libraries needed for handler functionality
- Must link against UIKit framework

### Android
- Handler renamed to `.so` for APK distribution
- Links against NDK C++ static libraries
- Requires specific API level compatibility

### Linux
- Standard ar/ranlib toolchain
- Links against system pthread and stdc++

### Windows
- MinGW or MSVC toolchain
- Handler must be distributed with .exe extension
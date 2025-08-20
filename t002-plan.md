# T-002 Windows Build Support - Refactoring Plan

## Overview

Current Windows build requires Python, MSVC, and Clang dependencies, defeating the purpose of removing depot_tools dependency. Since Python is unavoidable for Windows builds, we should provide multiple build strategies to let users choose based on their needs.

## Design Principles
- **YAGNI**: Only implement what's actually needed
- **KISS**: Minimal changes to existing structure  
- **DRY**: Reuse existing phases (wrapper build, bindgen)
- **SRP**: Each feature flag has single responsibility

## Build Strategies

### 1. `vendored` (default)
- **What**: Build from vendored Crashpad source using standalone tools
- **Platforms**: Linux, macOS, Android, iOS
- **Dependencies**: None (downloads GN/Ninja automatically)
- **Windows**: Not supported
- **Current Status**: Already implemented

### 2. `vendored-depot`
- **What**: Build Crashpad using depot_tools with official gclient workflow
- **Platforms**: All platforms including Windows
- **Dependencies**: Python (via depot_tools)
- **Use Case**: Windows builds, official Crashpad build process
- **Implementation**: 
  - Download depot_tools
  - Use gclient sync in temp directory (fresh checkout)
  - Build with official workflow
  - Copy artifacts back
- **Reference**: [Crashpad Official Build](https://chromium.googlesource.com/crashpad/crashpad/+/HEAD/doc/developing.md)

### 3. `prebuilt`
- **What**: Download pre-compiled Crashpad libraries from GitHub Releases
- **Platforms**: All platforms
- **Dependencies**: None
- **Use Case**: Fast builds, CI/CD, cargo install
- **Implementation**: Skip Crashpad build, download .a/.lib files

## Feature Flag Configuration

```toml
# Cargo.toml
[features]
default = ["vendored"]
vendored = []           # Existing standalone build
vendored-depot = []     # Use depot_tools for build
prebuilt = []          # Download pre-built libraries

# Mutually exclusive - enforced in build.rs
```

## Implementation Plan

### Phase 1: Prepare Infrastructure
- [ ] Add feature flags to `crashpad-sys/Cargo.toml`
- [ ] Add mutual exclusion check in `build.rs`
- [ ] Create `build/strategies.rs` module for build strategy selection

### Phase 2: Refactor Existing Code
- [ ] Extract current Windows error into `vendored` feature guard
- [ ] Move tool download logic to `vendored` specific path
- [ ] Ensure `phases.rs` compile/bind steps are feature-agnostic

### Phase 3: Implement vendored-depot
- [ ] Add `depot_tools_download()` function in `tools.rs`
- [ ] Create temp build directory for gclient workflow
- [ ] Generate proper `.gclient` file with `managed: True`
- [ ] Run `gclient sync` to fetch Crashpad and dependencies
- [ ] Copy our `crashpad_wrapper.cc` to temp build dir
- [ ] Set PATH to include depot_tools directory
- [ ] Run official build process (gn gen, ninja)
- [ ] Copy build artifacts back to OUT_DIR
- [ ] Clean up temp directory

### Phase 4: Implement prebuilt Download
- [ ] Create `build/prebuilt.rs` module
- [ ] Implement GitHub Release download logic
- [ ] Add version matching (crate version = release tag)
- [ ] Skip prepare/configure/compile phases
- [ ] Link downloaded libraries directly

### Phase 5: Implement prebuilt Generation
- [ ] Add `build-prebuilt` command to xtask
- [ ] Build crashpad using vendored-depot feature
- [ ] Package build artifacts (libcrashpad.a, headers)
- [ ] Create platform-specific archives
- [ ] Generate checksums for verification

### Phase 6: Testing & Documentation
- [ ] Test all three strategies on Linux/macOS
- [ ] Test vendored-depot on Windows
- [ ] Test prebuilt generation with xtask
- [ ] Update README with build strategy guide
- [ ] Create CI workflow for prebuilt artifact generation

## File Structure Changes

```
crashpad-sys/
  build.rs              # Add feature flag dispatch
  build/
    config.rs          # Modify Windows setup based on feature
    phases.rs          # Keep mostly unchanged (vendored path)
    tools.rs           # Add depot_tools download function
    depot_build.rs     # NEW: Complete depot_tools build workflow
    prebuilt.rs        # NEW: Prebuilt download logic

xtask/
  src/
    main.rs            # Add build-prebuilt command
    prebuilt.rs        # NEW: Prebuilt generation logic

.github/
  workflows/
    release.yml        # NEW: Auto-build prebuilt on tags
```

## Code Changes Overview

### build.rs
```rust
fn main() {
    // Feature flag validation
    #[cfg(all(feature = "vendored", feature = "vendored-depot"))]
    compile_error!("Only one build strategy can be selected");
    
    // Dispatch based on feature
    #[cfg(feature = "prebuilt")]
    return build::prebuilt::download_and_link();
    
    #[cfg(feature = "vendored-depot")]
    return build::depot_build::build_with_depot_tools();
    
    // vendored (default)
    #[cfg(feature = "vendored")]
    run().expect("Build failed");
}
```

### build/depot_build.rs - NEW
```rust
pub fn build_with_depot_tools() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Download depot_tools
    let depot_tools_dir = tools::ensure_depot_tools()?;
    env::set_var("PATH", format!("{};{}", depot_tools_dir.display(), env::var("PATH")?));
    
    // 2. Create temp directory
    let temp_dir = TempDir::new("crashpad-depot-build")?;
    
    // 3. Create .gclient
    let gclient_content = r#"
solutions = [{
    "name": "crashpad",
    "url": "https://chromium.googlesource.com/crashpad/crashpad.git",
    "managed": True,
}]"#;
    fs::write(temp_dir.path().join(".gclient"), gclient_content)?;
    
    // 4. gclient sync
    Command::new("gclient")
        .arg("sync")
        .current_dir(&temp_dir)
        .status()?;
    
    // 5. Copy our wrapper
    fs::copy(
        manifest_dir.join("crashpad_wrapper.cc"),
        temp_dir.path().join("crashpad/crashpad_wrapper.cc")
    )?;
    
    // 6. Build
    let build_dir = temp_dir.path().join("crashpad/out/Default");
    Command::new("gn")
        .args(&["gen", "out/Default", "--args", &gn_args])
        .current_dir(temp_dir.path().join("crashpad"))
        .status()?;
    
    Command::new("ninja")
        .args(&["-C", "out/Default"])
        .current_dir(temp_dir.path().join("crashpad"))
        .status()?;
    
    // 7. Copy artifacts to OUT_DIR
    copy_build_artifacts(&build_dir, &out_dir)?;
    
    // 8. TempDir auto-cleanup
    Ok(())
}
```

### config.rs - setup_windows()
```rust
fn setup_windows(&mut self, target: &str) -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(feature = "vendored")]
    return Err("Windows not supported with vendored feature. Use vendored-depot or prebuilt".into());
    
    #[cfg(feature = "vendored-depot")]
    {
        // Simple config - depot_tools handles complexity
        self.gn_args.insert("target_os".to_string(), "\"win\"".to_string());
        // Minimal args only
    }
}
```

### xtask/src/main.rs
```rust
enum Commands {
    // ... existing commands ...
    
    /// Build prebuilt packages for all platforms
    BuildPrebuilt {
        /// Target triple (optional, defaults to current)
        #[arg(long)]
        target: Option<String>,
    },
}

fn build_prebuilt(target: Option<String>) -> Result<()> {
    let target = target.unwrap_or_else(|| env::var("TARGET").unwrap());
    
    // 1. Build crashpad with vendored-depot
    println!("Building crashpad for {}", target);
    Command::new("cargo")
        .args(&["build", "-p", "crashpad-rs-sys", 
                "--features", "vendored-depot",
                "--target", &target])
        .status()?;
    
    // 2. Collect artifacts
    let out_dir = find_out_dir(&target)?;
    let dist_dir = Path::new("dist").join(&target);
    fs::create_dir_all(&dist_dir)?;
    
    // 3. Copy libraries and headers
    let libs = ["libcrashpad.a", "libclient.a", "libutil.a", /* ... */];
    for lib in &libs {
        let src = out_dir.join("crashpad_build").join(lib);
        let dst = dist_dir.join("lib").join(lib);
        fs::copy(src, dst)?;
    }
    
    // Copy wrapper.h for bindgen
    fs::copy("crashpad-sys/wrapper.h", dist_dir.join("include/wrapper.h"))?;
    
    // 4. Create tarball
    let archive_name = format!("crashpad-{}-{}.tar.gz", 
                              env!("CARGO_PKG_VERSION"), target);
    create_tarball(&dist_dir, &archive_name)?;
    
    // 5. Generate checksum
    generate_checksum(&archive_name)?;
    
    println!("Created prebuilt package: {}", archive_name);
    Ok(())
}
```

### .github/workflows/release.yml
```yaml
name: Build Prebuilt Packages

on:
  push:
    tags:
      - 'v*'

jobs:
  build:
    strategy:
      matrix:
        include:
          - os: ubuntu-latest
            target: x86_64-unknown-linux-gnu
          - os: macos-latest
            target: x86_64-apple-darwin
          - os: macos-latest
            target: aarch64-apple-darwin
          - os: windows-latest
            target: x86_64-pc-windows-msvc
    
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v3
        with:
          submodules: recursive
      
      - name: Install depot_tools (Windows)
        if: matrix.os == 'windows-latest'
        run: |
          git clone https://chromium.googlesource.com/chromium/tools/depot_tools.git
          echo "DEPOT_TOOLS_PATH=$PWD/depot_tools" >> $GITHUB_ENV
      
      - name: Build prebuilt package
        run: cargo xtask build-prebuilt --target ${{ matrix.target }}
      
      - name: Upload to Release
        uses: softprops/action-gh-release@v1
        with:
          files: dist/*.tar.gz
```

## Directory Structure for Prebuilt
```
dist/
  x86_64-unknown-linux-gnu/
    lib/
      libcrashpad.a
      libclient.a
      libutil.a
      ...
    include/
      wrapper.h
  x86_64-pc-windows-msvc/
    lib/
      crashpad.lib
      client.lib
      ...
    include/
      wrapper.h
```

## Success Criteria
1. Linux/macOS builds work with `vendored` (existing behavior)
2. Windows builds work with `vendored-depot`  
3. All platforms work with `prebuilt`
4. `cargo xtask build-prebuilt` generates valid packages
5. CI automatically creates releases with prebuilt packages
6. No breaking changes to existing API
7. Clear error messages for unsupported combinations

## Risks & Mitigations
- **Risk**: depot_tools download might fail
  - **Mitigation**: Clear error message with manual download instructions
- **Risk**: prebuilt versions might not match
  - **Mitigation**: Strict version matching, fallback to build from source
- **Risk**: xtask build-prebuilt might have platform issues
  - **Mitigation**: Test on CI for all platforms before release

## User Experience

```bash
# Default - Linux/macOS only
cargo build

# Windows support
cargo build --features vendored-depot --no-default-features

# Fast installation (all platforms)
cargo build --features prebuilt --no-default-features

# Generate prebuilt packages (maintainers)
cargo xtask build-prebuilt --target x86_64-pc-windows-msvc
```

## Timeline
- Phase 1-2: 1 hour (refactoring)
- Phase 3: 2 hours (depot_tools integration)
- Phase 4: 2 hours (prebuilt download)
- Phase 5: 3 hours (prebuilt generation + xtask)
- Phase 6: 1 hour (testing & docs)

Total: ~9 hours of implementation
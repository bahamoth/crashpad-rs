<!-- DOCGUIDE HEADER
Version: 1.0
Generated: 2025-08-06
Project Type: Rust FFI Library
Primary Language: Rust
Last Updated: 2025-08-06
Update Command: claude commands/scaffold/conventions.md
-->

# Development Conventions

This document defines the development standards and practices for the crashpad-rs project.

## Quick Policy Matrix

| Category | Tool | Config | Enforcement | CI Check |
|----------|------|--------|-------------|----------|
| Code Format | rustfmt | Default | Manual | ⚠️ TODO |
| Linting | clippy | Default | Manual | ⚠️ TODO |
| Testing | cargo nextest | Cargo.toml | Manual | ✅ |
| Commits | Conventional | Manual | None | ⚠️ TODO |
| Dependencies | cargo/gclient | build.rs | Build step | ✅ |

## Code Style

### General Principles
- **Rust Wrapper**: Prefer safe Rust abstractions over unsafe FFI
- **Clear Boundaries**: Separate sys crate (FFI) from safe wrapper
- **Cross-Platform**: Test on all supported platforms before merge
- **Build Reproducibility**: All dependencies managed through build.rs

### Rust Conventions

#### Formatting (rustfmt)
```bash
# Format all Rust code
cargo fmt --all

# Check formatting without changes
cargo fmt --all -- --check
```

#### Linting (clippy)
```bash
# Run clippy with all targets
cargo clippy --all-targets --all-features -- -D warnings

# Fix clippy warnings automatically
cargo clippy --fix
```

#### Code Organization
- `crashpad-sys/`: Raw FFI bindings only, minimal safe wrappers
- `crashpad/`: Safe Rust API, handles all unsafe interactions
- `xtask/`: Build tooling and automation
- Use `thiserror` for error types (workspace dependency)

#### FFI Safety Rules
- All unsafe code must have safety comments
- Prefer `CString` over raw pointers for strings
- Check null pointers before dereferencing
- Use `std::mem::ManuallyDrop` for C++ managed resources

### C++ Conventions

#### Files
- `wrapper.h`: C API declarations
- `crashpad_wrapper.cc`: C++ to C bridge implementation

#### Naming
- C functions: `crashpad_` prefix (e.g., `crashpad_init`)
- Opaque types: Forward declarations only in header
- Resource management: Always provide cleanup functions

## Git Conventions

### Branch Naming
- Feature: `feature/description-kebab-case`
- Fix: `fix/issue-description`
- Refactor: `refactor/component-name`

### Commit Messages
Follow Conventional Commits format:

```
<type>(<scope>): <subject>

[optional body]

[optional footer]
```

Types:
- `feat`: New feature (e.g., platform support)
- `fix`: Bug fix
- `refactor`: Code restructuring
- `build`: Changes to build.rs or Makefile
- `test`: Adding or updating tests
- `docs`: Documentation changes
- `chore`: Maintenance tasks

Examples:
- `feat(android): add Android NDK cross-compilation support`
- `fix(ios): resolve in-process handler initialization`
- `build: add depot_tools auto-download in build.rs`
- `refactor: simplify Makefile platform detection`

### Merge Strategy
- **Prefer rebase**: Keep linear history
- **Fast-forward when possible**: No merge commits for simple changes
- **Squash small commits**: One logical change per commit
- **Delete branches after merge**: Keep repository clean

## Build & Packaging

### Standard Commands
```bash
# Development build
cargo build --package crashpad-sys
cargo build --package crashpad

# Release build
make build-release

# Clean build (removes native artifacts)
make clean
cargo build

# Run tests
cargo test --lib  # Unit tests only
cargo nextest run --test '*'  # Integration tests with isolation

# Create distribution package
make dist
```

## Cross-Compilation

### Android

#### Prerequisites
1. Install Android NDK from [Android NDK Downloads](https://developer.android.com/ndk/downloads)
2. Set environment variable:
   ```bash
   export ANDROID_NDK_HOME=/path/to/android-ndk
   ```
3. **IMPORTANT for NDK r22+**: Create symlinks for compatibility:
   ```bash
   cd $ANDROID_NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin
   ln -sf llvm-ar aarch64-linux-android-ar
   ln -sf llvm-ar arm-linux-androideabi-ar
   ln -sf llvm-ar i686-linux-android-ar
   ln -sf llvm-ar x86_64-linux-android-ar
   ```

#### Building with cargo-ndk (Recommended)
```bash
# Install cargo-ndk
cargo install cargo-ndk

# Add Rust targets
rustup target add aarch64-linux-android
rustup target add armv7-linux-androideabi
rustup target add x86_64-linux-android
rustup target add i686-linux-android

# Build for specific architectures
cargo ndk -t arm64-v8a build --package crashpad-sys    # ARM64 (most devices)
cargo ndk -t armeabi-v7a build --package crashpad-sys  # ARMv7 (older devices)
cargo ndk -t x86_64 build --package crashpad-sys       # x86_64 (emulators)
cargo ndk -t x86 build --package crashpad-sys          # x86 (older emulators)
```

#### Manual Build (Alternative)
```bash
# Add NDK tools to PATH
export PATH=$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin:$PATH

# Build
cargo build --package crashpad-sys --target aarch64-linux-android
```

#### Troubleshooting Android Builds
- **"error adding symbols: file in wrong format"**: Build only the library with `--package crashpad-sys`
- **"llvm-ar: not found"**: Ensure NDK bin directory is in PATH or create symlinks
- **"ANDROID_NDK_HOME not set"**: Export the environment variable

### iOS

#### Prerequisites
- macOS with Xcode installed
- iOS SDK available

#### Building
```bash
# Add iOS targets
rustup target add aarch64-apple-ios
rustup target add aarch64-apple-ios-sim
rustup target add x86_64-apple-ios  # For older simulators

# Build for iOS device
cargo build --target aarch64-apple-ios

# Build for iOS simulator (Apple Silicon)
cargo build --target aarch64-apple-ios-sim

# Build for iOS simulator (Intel)
cargo build --target x86_64-apple-ios

# Run iOS simulator example
cargo build --target aarch64-apple-ios-sim --example ios_simulator_test
```

### Windows Cross-Compilation (from Linux/WSL)

#### Setup
```bash
# Ubuntu/Debian
sudo apt install mingw-w64

# Fedora
sudo dnf install mingw64-gcc

# Add Windows target
rustup target add x86_64-pc-windows-gnu
```

#### Building
```bash
cargo build --target x86_64-pc-windows-gnu
```

### General Cross-Compilation Tips
- The `.cargo/config.toml` file sets up correct tools for Android targets
- For CI/CD, ensure appropriate toolchains are installed
- When in doubt, use cargo-ndk for Android builds
- iOS builds require macOS; no Linux cross-compilation available

### Build Process(Internal)
1. `build.rs` clones depot_tools to `third_party/`
2. Creates `.gclient` configuration
3. Runs `gclient sync` for Crashpad dependencies
4. Generates build files with `gn`
5. Builds with `ninja`
6. `bindgen` creates FFI bindings from `wrapper.h`

## Testing Requirements

### Test Categories
- **Unit Tests**: Pure Rust logic in crashpad crate
- **Integration Tests**: FFI boundary testing
- **Platform Tests**: OS-specific functionality
- **Example Programs**: Demonstration and validation

### Running Tests
```bash
# Unit tests only (fast)
cargo test --lib

# Integration tests with process isolation
cargo nextest run --test '*'

# All tests
cargo nextest run

# Specific package
cargo nextest run -p crashpad

# With output
cargo test --lib -- --nocapture

# Platform-specific
cargo build --target aarch64-apple-ios-sim --example ios_simulator_test
```

### Coverage Goals
- Core API: 80% coverage
- Error handling: 100% coverage
- Platform-specific: Tested on target platform

## Dependencies

### Rust Dependencies
- Managed through `Cargo.toml`
- Workspace dependencies in root `Cargo.toml`
- Use exact versions for sys crate dependencies

### Native Dependencies
- Managed by `build.rs`
- Downloaded to `third_party/` (gitignored)
- Chromium build tools (depot_tools, gclient, gn, ninja)
- Never commit third_party content

### Update Policy
- Security updates: Immediate
- Bug fixes: Within sprint
- Feature updates: Quarterly review
- Breaking changes: Major version bump

## Documentation

### Code Documentation
```rust
/// Brief description (one line)
///
/// Detailed explanation if needed.
/// 
/// # Arguments
/// 
/// * `param` - Description
/// 
/// # Returns
/// 
/// Description of return value
/// 
/// # Errors
/// 
/// When this function returns errors
/// 
/// # Safety
/// 
/// Conditions for safe usage (unsafe functions only)
/// 
/// # Examples
/// 
/// ```no_run
/// use crashpad::Client;
/// let client = Client::new(config)?;
/// ```
pub fn function_name(param: Type) -> Result<ReturnType> {
    // Implementation
}
```

### Project Documentation
- `README.md`: Quick start and usage
- `ARCHITECTURE.md`: System design
- `CLAUDE.md`: AI assistant context
- `OVERVIEW.md`: Project overview and quick reference
- `CONVENTIONS.md`: This document

### API Documentation
```bash
# Generate and open docs
cargo doc --open

# Include dependencies
cargo doc --no-deps --open
```

## Security & Compliance

### Security Practices
- No secrets in code or commits
- Validate all FFI inputs
- Handle panics at FFI boundary
- Clear sensitive data from memory

### Dependency Scanning
```bash
# Check for known vulnerabilities
cargo audit

# Update vulnerable dependencies
cargo update
cargo audit fix
```

## Pre-Release Checklist

- [ ] Code formatted (`cargo fmt --all`)
- [ ] Clippy passes (`cargo clippy --all-targets`)
- [ ] Tests pass (`cargo test --lib && cargo nextest run --test '*'`)
- [ ] Documentation updated (`cargo doc`)
- [ ] Cross-platform builds verified
- [ ] CHANGELOG updated
- [ ] Version bumped (if needed)

## Platform-Specific Notes

### iOS
- Uses in-process handler (no separate executable)
- Simulator requires different target architecture
- Test with `examples/ios_simulator_test.rs`

### Android
- Requires Android NDK
- Set `ANDROID_NDK_HOME` environment variable
- Limited to library builds (no handler executable)

### Windows
- Cross-compilation supported from Unix
- Native compilation requires Visual Studio
- Handler executable deployment critical

## Integration

### Overview
Applications using crashpad-rs require the `crashpad_handler` executable at runtime. The handler captures crash dumps and uploads them to your crash reporting server.

### Development vs Production

#### Development (Debug Builds)
- Build script automatically sets `CRASHPAD_HANDLER` environment variable
- Handler located in `third_party/crashpad_checkout/crashpad/out/{platform}/`
- Examples and tests work out of the box

#### Production (Release Builds)
- Environment variables NOT automatically set
- Handler must be distributed with application
- Explicit configuration required

### Distribution Steps

1. **Build for production**:
   ```bash
   cargo build --release
   # Or use xtask for complete package
   cargo xtask dist
   ```

2. **Locate the handler**:
   - From `xtask dist`: `dist/bin/crashpad_handler`
   - From build directory: `third_party/crashpad_checkout/crashpad/out/{platform}/crashpad_handler`
   - Platform names: `crashpad_handler` (Unix), `crashpad_handler.exe` (Windows)

3. **Package structure**:
   ```
   my-app/
   ├── my-app              # Your application executable
   ├── crashpad_handler    # Handler executable (required)
   └── lib/                # Any required dynamic libraries
   ```

### Handler Location Options

1. **Environment Variable**:
   ```bash
   CRASHPAD_HANDLER=/path/to/crashpad_handler ./my-app
   ```

2. **Same Directory** (recommended):
   - Place handler in same directory as application
   - Library automatically searches here

3. **Custom Path** (programmatic):
   ```rust
   use crashpad::CrashpadConfig;
   
   let config = CrashpadConfig::builder()
       .handler_path("/opt/myapp/bin/crashpad_handler")
       .build();
   ```

### Platform-Specific Integration

#### Linux
- Ensure execute permissions: `chmod +x crashpad_handler`
- Check dependencies: `ldd crashpad_handler`
- Systemd service example:
  ```ini
  [Unit]
  Description=My Application
  After=network.target
  
  [Service]
  Type=simple
  ExecStart=/opt/myapp/bin/my-app
  WorkingDirectory=/opt/myapp
  Environment="CRASHPAD_HANDLER=/opt/myapp/bin/crashpad_handler"
  Restart=on-failure
  
  [Install]
  WantedBy=multi-user.target
  ```

#### macOS
- Code sign handler for distribution
- Consider notarization for macOS 10.15+
- Bundle frameworks if needed

#### iOS
- Handler runs in-process (no separate executable)
- No special integration steps needed

#### Windows
- Distribute Visual C++ Redistributables if needed
- Place handler in same directory as .exe
- Check with Dependency Walker for missing DLLs


#### Android
- Bundle handler as asset or native library
- Special APK packaging considerations


### Verification

1. **Check handler is found**:
   ```rust
   println!("Using handler at: {:?}", config.handler_path);
   ```

2. **Test crash reporting** (Unix):
   ```bash
   kill -SIGSEGV $(pidof my-app)
   ```

3. **Verify uploads**:
   - Check crash server for reports
   - Ensure network connectivity

### Troubleshooting

#### Handler Not Found
- Check file exists and permissions
- Verify environment variables
- Use `strace`/`dtruss` to trace file access

#### Crashes Not Captured
- Ensure handler starts successfully
- Check database path permissions
- Verify architecture match (handler and app)

#### Upload Failures
- Test network connectivity
- Check firewall rules
- Verify server URL and credentials

### Security Considerations

- Handler runs with same privileges as application
- Crash dumps may contain memory contents
- Use HTTPS for uploads
- Consider encrypting local storage
- Implement retention policies

### Best Practices

1. Always test crash reporting in deployment environment
2. Monitor handler process health
3. Secure crash dumps (may contain sensitive data)
4. Implement retry logic for failed uploads
5. Document handler location in application README

## Related Documents

- [PRD.md](./PRD.md) - Product requirements
- [ARCHITECTURE.md](./ARCHITECTURE.md) - Technical design
- [OVERVIEW.md](./OVERVIEW.md) - Project overview
- [TASKS.md](./TASKS.md) - Work tracking
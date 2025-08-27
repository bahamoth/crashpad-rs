# crashpad-rs

![Version](https://img.shields.io/github/v/release/bahamoth/crashpad-rs?color=blue)
[![docs.rs](https://docs.rs/crashpad-rs/badge.svg)](https://docs.rs/crashpad-rs)
[![crates.io](https://img.shields.io/crates/d/crashpad-rs.svg)](https://crates.io/crates/crashpad-rs)
[![Build Status](https://github.com/bahamoth/crashpad-rs/actions/workflows/test-android.yml/badge.svg)](https://github.com/bahamoth/crashpad-rs/actions/workflows/test-android.yml)
[![Build Status](https://github.com/bahamoth/crashpad-rs/actions/workflows/test-ios.yml/badge.svg)](https://github.com/bahamoth/crashpad-rs/actions/workflows/test-ios.yml)
[![Build Status](https://github.com/bahamoth/crashpad-rs/actions/workflows/test-macos.yml/badge.svg)](https://github.com/bahamoth/crashpad-rs/actions/workflows/test-macos.yml)
[![Build Status](https://github.com/bahamoth/crashpad-rs/actions/workflows/test-linux.yml/badge.svg)](https://github.com/bahamoth/crashpad-rs/actions/workflows/test-linux.yml)
[![Build Status](https://github.com/bahamoth/crashpad-rs/actions/workflows/test-windows.yml/badge.svg)](https://github.com/bahamoth/crashpad-rs/actions/workflows/test-windows.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Rust bindings for Google Crashpad crash reporting system.

## Project Structure

| Directory       | Package Name      | Description                                    |
|-----------------|-------------------|------------------------------------------------|
| `crashpad-sys/` | `crashpad-rs-sys` | Low-level FFI bindings to Crashpad C++ library |
| `crashpad/`     | `crashpad-rs`     | Safe Rust wrapper API                          |

**Note**: The directories are published with different names to avoid conflicts on crates.io:

- `crashpad-sys/` → `crashpad-rs-sys`
- `crashpad/` → `crashpad-rs`

## Features

- **Cross-platform**: macOS, Linux, iOS, Android
- **Safe API**: Rust-safe wrapper around Crashpad C++ library
- **Flexible Configuration**: Runtime handler configuration
- **(Native)Crash Handler Included**: Native Handler executable built-in

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
# x-release-please-start-version
crashpad-rs = "0.2.6"
# x-release-please-end-version
```

## Quick Start

```rust
use crashpad_rs::{CrashpadClient, CrashpadConfig};
use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the Crashpad client
    let client = CrashpadClient::new()?;

    // Configure crash reporting
    let config = CrashpadConfig::builder()
        .handler_path("/path/to/crashpad_handler")  // Handler executable
        .database_path("./crashes")                 // Local crash storage
        .url("https://your-crash-server.com/submit") // Upload endpoint
        .build();

    // Add application metadata
    let mut annotations = HashMap::new();
    annotations.insert("version".to_string(), env!("CARGO_PKG_VERSION").to_string());
    annotations.insert("build".to_string(), "release".to_string());

    // Start crash monitoring
    client.start_with_config(&config, &annotations)?;

    // Your application code here
    Ok(())
}
```

## Configuration

### Basic Configuration (Local Only)

```rust
// Minimal config - crashes saved locally, no upload
let config = CrashpadConfig::builder()
.handler_path("./crashpad_handler")  // Required on desktop platforms
.database_path("./crash_dumps")      // Where to store crash dumps
.build();
```

### Production Configuration

```rust
// Full configuration with upload server
let config = CrashpadConfig::builder()
.handler_path(std::env::var("CRASHPAD_HANDLER")
.unwrap_or_else( | _ | "./crashpad_handler".to_string()))
.database_path("/var/crash/myapp")
.metrics_path("/var/metrics/myapp")  // Optional: metrics storage
.url("https://crashes.example.com/api/minidump")
.build();
```

### Platform-Specific Configuration

#### Desktop (macOS/Linux/Windows)

```rust
// Handler path is required for desktop platforms
let config = CrashpadConfig::builder()
.handler_path("./crashpad_handler")  // Must point to handler executable
.database_path("./crashes")
.build();
```

#### iOS/tvOS/watchOS

```rust
// iOS uses in-process handler - no handler_path needed
let config = CrashpadConfig::builder()
.database_path("./crashes")  // Relative to app's Documents directory
.url("https://crashes.example.com/api/minidump")
.build();
```

#### Android

```rust
// Android can use external handler (as .so file in APK)
let config = CrashpadConfig::builder()
.handler_path("./libcrashpad_handler.so")  // Renamed for APK distribution
.database_path("/data/data/com.example.app/crashes")
.build();
```

### Environment-Based Configuration

```rust
// Adjust configuration based on environment
let config = if cfg!(debug_assertions) {
// Development: local storage only, no rate limiting
CrashpadConfig::builder()
.handler_path("./target/debug/crashpad_handler")
.database_path("./dev_crashes")
.rate_limit(false)  // Disable rate limiting for testing
.build()
} else {
// Production: with upload server
CrashpadConfig::builder()
.handler_path("/usr/local/bin/crashpad_handler")
.database_path("/var/crash/myapp")
.url("https://crashes.example.com/submit")
.upload_gzip(true)  // Enable compression (default)
.build()
};
```

### Handler Arguments Configuration

```rust
// Control handler behavior with high-level API
let config = CrashpadConfig::builder()
.database_path("./crashes")
.rate_limit(false)              // Disable upload rate limiting
.upload_gzip(false)             // Disable gzip compression
.periodic_tasks(false)          // Disable periodic maintenance
.identify_client_via_url(false) // Don't add client ID to URL
.build();

// Advanced: use low-level API for custom arguments
let config = CrashpadConfig::builder()
.database_path("./crashes")
.handler_argument("--monitor-self")  // Enable self-monitoring
.handler_argument("--monitor-self-annotation=version=1.0")
.build();

// Mix high-level and low-level APIs
let config = CrashpadConfig::builder()
.database_path("./crashes")
.rate_limit(false)  // High-level API
.handler_argument("--monitor-self")  // Low-level API
.build();
```

**Note**: Handler arguments are currently ignored on iOS/tvOS/watchOS as they use an in-process handler with hardcoded
settings. This may change in future Crashpad versions.

## Platform Support

| Platform | Architecture            | Status   | Handler Type        |
|----------|-------------------------|----------|---------------------|
| macOS    | x86_64, aarch64         | ✅ Stable | External executable |
| Linux    | x86_64, aarch64         | ✅ Stable | External executable |
| iOS      | arm64, x86_64 sim       | ✅ Stable | In-process          |
| Android  | arm, arm64, x86, x86_64 | ✅ Stable | External/In-process |
| Windows  | x86_64                  | ✅ Stable | External executable |

## Advanced Features

### Capturing Dumps Without Crashing

The `dump_without_crash()` method allows you to capture diagnostic information without terminating your application. This is useful for:
- Debugging production issues
- Capturing state during recoverable errors
- Performance monitoring
- User-requested diagnostics

```rust
use crashpad_rs::{CrashpadClient, CrashpadConfig};

// Initialize Crashpad as usual
let client = CrashpadClient::new()?;
let config = CrashpadConfig::builder()
    .handler_path("./crashpad_handler")
    .database_path("./crashes")
    .build();
client.start_with_config(&config, &Default::default())?;

// Capture diagnostic dump on error condition
if let Err(e) = some_operation() {
    // Log the error
    eprintln!("Operation failed: {}", e);
    
    // Capture current state for analysis
    client.dump_without_crash();
    
    // Continue running - no crash occurs
    handle_error_gracefully(e);
}
```

## Examples

### Running the Test Example

1. **Build the example and handler**
   ```bash
   # Build everything including the handler
   cargo build --example crashpad_test_cli
   
   # The handler will be at: target/debug/crashpad_handler
   # The example will be at: target/debug/examples/crashpad_test_cli
   ```

2. **Run with handler in same directory** (easiest)
   ```bash
   # Copy handler to current directory
   cp target/debug/crashpad_handler .
   
   # Run the example (will look for handler in current directory as fallback)
   ./target/debug/examples/crashpad_test_cli
   ```

3. **Run with environment variable**
   ```bash
   # Set handler path explicitly
   export CRASHPAD_HANDLER=target/debug/crashpad_handler
   
   # Run from anywhere
   cargo run --example crashpad_test_cli
   ```

4. **Run directly with cargo** (if handler is in PATH or current directory)
   ```bash
   # If you have crashpad_handler in current directory or PATH
   cargo run --example crashpad_test_cli
   ```

5. **Test the available commands**
   ```bash
   # Show help
   cargo run --example crashpad_test_cli -- --help
   
   # Capture a diagnostic dump without crashing
   cargo run --example crashpad_test_cli -- dump
   
   # Trigger a real crash for testing
   cargo run --example crashpad_test_cli -- crash
   
   # Run automated tests
   cargo run --example crashpad_test_cli -- test
   ```

**Note**: The example looks for the handler in this order:

1. Path specified in config (if provided)
2. `CRASHPAD_HANDLER` environment variable
3. Same directory as the executable (fallback)
4. Current working directory (fallback)

### Handler Deployment

The `crashpad_handler` executable must be available at runtime. Common approaches:

1. **Same directory as application** (simplest)
   ```
   my-app/
   ├── my-app
   └── crashpad_handler
   ```

2. **System path** (for installed applications)
   ```
   /usr/local/bin/crashpad_handler
   ```

3. **Environment variable** (flexible deployment)
   ```bash
   export CRASHPAD_HANDLER=/opt/myapp/bin/crashpad_handler
   ```

4. **Bundled in package** (platform-specific)
    - macOS: Inside .app bundle
    - Linux: In AppImage or snap
    - Android: As .so file in APK
    - iOS: Not needed (in-process)

## Documentation

### For Library Users

- [API Documentation](https://docs.rs/crashpad-rs) - Full API reference
- [Integration Guide](https://github.com/bahamoth/crashpad-rs/blob/main/CONVENTIONS.md#integration) - Production
  integration

### For Contributors

- [Development Guide](https://github.com/bahamoth/crashpad-rs/blob/main/DEVELOPING.md) - Build, test, and contribute
- [Architecture](https://github.com/bahamoth/crashpad-rs/blob/main/ARCHITECTURE.md) - Technical design decisions
- [Conventions](https://github.com/bahamoth/crashpad-rs/blob/main/CONVENTIONS.md) - Coding standards

## Known Limitations

- **Windows Support**: Not currently available (build system limitation)
- **iOS Handler Arguments**: Handler arguments are ignored on iOS/tvOS/watchOS as the in-process handler uses hardcoded
  settings (Crashpad limitation, see [bug #23](https://crashpad.chromium.org/bug/23))
- **Handler Update**: No automatic update mechanism for deployed handlers

Contributions are welcome!

## Troubleshooting

### Handler Not Found

- Verify handler executable exists and has execute permissions
- Check `CRASHPAD_HANDLER` environment variable
- Explicitly set path in config: `.handler_path("/path/to/crashpad_handler")`
- Ensure handler architecture matches application

### Crashes Not Being Captured

- Confirm handler process is running
- Check database path has write permissions
- Verify network connectivity for uploads

## License

Licensed under MIT license ([LICENSE](LICENSE)).

## Contributing

Contributions are welcome! See [DEVELOPING.md](https://github.com/bahamoth/crashpad-rs/blob/main/DEVELOPING.md) for
build and test instructions.

## Support

- **Issues**: [GitHub Issues](https://github.com/bahamoth/crashpad-rs/issues)
- **Discussions**: [GitHub Discussions](https://github.com/bahamoth/crashpad-rs/discussions)

# crashpad-rs

Rust bindings for Google Crashpad crash reporting system.

## Project Structure

| Directory | Package Name | Description |
|-----------|-------------|------------|
| `crashpad-sys/` | `crashpad-rs-sys` | Low-level FFI bindings to Crashpad C++ library |
| `crashpad/` | `crashpad` | Safe Rust wrapper API |
| `xtask/` | - | Development automation (not published) |

**Note**: The `crashpad-sys` directory is published as `crashpad-rs-sys` to avoid naming conflicts on crates.io.

## Features

- **Cross-platform**: macOS, Linux, Windows, iOS, Android
- **Safe API**: Rust-safe wrapper around Crashpad C++ library
- **Flexible Configuration**: Runtime handler configuration
- **Zero-copy**: Minimal overhead for crash reporting
- **Production Ready**: Battle-tested crash reporting solution

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
crashpad = "0.1"
```

## Quick Start

```rust
use crashpad::{CrashpadClient, CrashpadConfig};
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
        .unwrap_or_else(|_| "./crashpad_handler".to_string()))
    .database_path("/var/crash/myapp")
    .metrics_path("/var/metrics/myapp")  // Optional: metrics storage
    .url("https://crashes.example.com/api/minidump")
    .rate_limit(true)  // Limit upload frequency
    .compress_uploads(true)  // Compress before uploading
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
    // Development: local storage only
    CrashpadConfig::builder()
        .handler_path("./target/debug/crashpad_handler")
        .database_path("./dev_crashes")
        .build()
} else {
    // Production: with upload server
    CrashpadConfig::builder()
        .handler_path("/usr/local/bin/crashpad_handler")
        .database_path("/var/crash/myapp")
        .url("https://crashes.example.com/submit")
        .build()
};
```

## Platform Support

| Platform | Architecture | Status | Handler Type |
|----------|-------------|--------|--------------|
| macOS | x86_64, aarch64 | ✅ Stable | External executable |
| Linux | x86_64, aarch64 | ✅ Stable | External executable |
| Windows | x86_64 | ✅ Stable | External executable |
| iOS | arm64, x86_64 sim | ✅ Stable | In-process |
| Android | arm, arm64, x86, x86_64 | ✅ Stable | External/In-process |

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
- [API Documentation](https://docs.rs/crashpad) - Full API reference
- [Integration Guide](CONVENTIONS.md#integration) - Production integration
- [Platform Notes](CONVENTIONS.md#platform-specific-integration) - Platform-specific considerations

### For Contributors
- [Development Guide](DEVELOPING.md) - Build, test, and contribute
- [Architecture](ARCHITECTURE.md) - Technical design decisions
- [Conventions](CONVENTIONS.md) - Coding standards

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

Contributions are welcome! See [DEVELOPING.md](DEVELOPING.md) for build and test instructions.

## Support

- **Issues**: [GitHub Issues](https://github.com/bahamoth/crashpad-rs/issues)
- **Discussions**: [GitHub Discussions](https://github.com/bahamoth/crashpad-rs/discussions)
- **Security**: Report security vulnerabilities to security@example.com
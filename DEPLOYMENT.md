# Deployment Guide

This guide explains how to properly deploy applications using crashpad-rs.

## Overview

When deploying an application that uses crashpad-rs, you need to ensure the `crashpad_handler` executable is available at runtime. The handler is responsible for capturing crash dumps and uploading them to your crash reporting server.

## Development vs Production

### Development (Debug Builds)
- The build script automatically sets `CRASHPAD_HANDLER` environment variable
- Examples and tests work out of the box with `cargo run`
- Handler is located in `third_party/crashpad_checkout/crashpad/out/{platform}/`

### Production (Release Builds)
- Environment variables are NOT automatically set
- You must distribute the handler with your application
- Handler must be explicitly configured or placed in expected locations

## Distribution Steps

### 1. Build Your Application

```bash
# Build in release mode
cargo build --release

# Or use xtask for a complete distribution package
cargo xtask dist
```

### 2. Locate the Handler

After building, the handler executable is available at:
- **From xtask dist**: `dist/bin/crashpad_handler`
- **From build directory**: `third_party/crashpad_checkout/crashpad/out/{platform}/crashpad_handler`

Platform-specific names:
- Linux/macOS: `crashpad_handler`
- Windows: `crashpad_handler.exe`

### 3. Package Your Application

Your distribution should include:
```
my-app/
├── my-app              # Your application executable
├── crashpad_handler    # The handler executable (same directory)
└── lib/                # Any required dynamic libraries
```

### 4. Handler Location Options

The crashpad-rs library searches for the handler in this order:

1. **Environment Variable** (if set)
   ```bash
   CRASHPAD_HANDLER=/path/to/crashpad_handler ./my-app
   ```

2. **Same Directory** (automatic fallback)
   - Place `crashpad_handler` in the same directory as your application
   - Library automatically looks here if no path is specified
   - This is the most portable approach

3. **Custom Path** (programmatic)
   ```rust
   use crashpad::CrashpadConfig;
   
   let config = CrashpadConfig::builder()
       .handler_path("/opt/myapp/bin/crashpad_handler")
       .build();
   ```

## Platform-Specific Notes

### Linux
- Ensure handler has execute permissions: `chmod +x crashpad_handler`
- May require additional libraries (check with `ldd crashpad_handler`)

### macOS
- Handler must be code-signed for distribution
- Consider notarization requirements for macOS 10.15+
- Framework dependencies are typically bundled

### Windows
- Distribute Visual C++ Redistributables if needed
- Handler should be in the same directory as your .exe

### Mobile Platforms

#### iOS
- Handler runs in-process (no separate executable needed)

#### Android
- Handler can be bundled as an asset or native library
- Special considerations for APK packaging

## Docker Deployment

```dockerfile
FROM debian:bullseye-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    libstdc++6 \
    && rm -rf /var/lib/apt/lists/*

# Copy application and handler
COPY --from=builder /app/target/release/my-app /usr/local/bin/
COPY --from=builder /app/dist/bin/crashpad_handler /usr/local/bin/

# Ensure handler is executable
RUN chmod +x /usr/local/bin/crashpad_handler

CMD ["my-app"]
```

## Systemd Service

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

## Verification

Test your deployment:

1. **Check handler is found**:
   ```rust
   // Your application should log handler location on startup
   println!("Using handler at: {:?}", config.handler_path);
   ```

2. **Test crash reporting**:
   ```bash
   # Send SIGSEGV to test crash handling (Unix)
   kill -SIGSEGV $(pidof my-app)
   ```

3. **Verify uploads**:
   - Check your crash reporting server for incoming reports
   - Ensure network connectivity from deployment environment

## Troubleshooting

### Handler Not Found
- Check file exists and has correct permissions
- Verify environment variables
- Use `strace` or `dtruss` to see file access attempts

### Crashes Not Captured
- Ensure handler starts successfully
- Check database path permissions
- Verify handler and application architecture match

### Upload Failures
- Test network connectivity to crash server
- Check firewall rules
- Verify crash server URL and credentials

## Best Practices

1. **Always test crash reporting** in your deployment environment
2. **Monitor handler process** - it should remain running
3. **Secure your crash dumps** - they may contain sensitive data
4. **Implement retry logic** for failed uploads
5. **Document handler location** in your application's README

## Security Considerations

- Handler runs with same privileges as your application
- Crash dumps may contain memory contents
- Use HTTPS for crash report uploads
- Consider encrypting local crash dump storage
- Implement appropriate retention policies
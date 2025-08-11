# Packaging Instructions

## Including crashpad_handler in your application

When distributing your application, you need to include the `crashpad_handler` executable.

### Option 1: Use cargo xtask dist

Build a distribution package with all necessary files:

```bash
cargo xtask dist
```

This creates a `dist/` directory with:
- `bin/crashpad_handler` - The handler executable
- `lib/` - Rust libraries
- `include/` - C API headers

### Option 2: Cargo Package Metadata

Add to your `Cargo.toml`:

```toml
[package.metadata.bundle]
resources = ["crashpad_handler"]
```

### Option 3: Manual Packaging

After building, copy the handler from:
- `third_party/crashpad_checkout/crashpad/out/{platform}/crashpad_handler`

To your distribution package, placing it next to your executable.

## Usage in Your Application

```rust
use crashpad_rs::{CrashpadClient, CrashpadConfig, HandlerStrategy};
use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = CrashpadClient::new()?;
    
    // Use bundled handler (looks next to executable)
    let config = CrashpadConfig::builder()
        .handler_strategy(HandlerStrategy::Bundled)
        .database_path("./crashes")
        .url("https://your-crash-server.com/submit")
        .build();
    
    let mut annotations = HashMap::new();
    annotations.insert("version".to_string(), env!("CARGO_PKG_VERSION").to_string());
    
    client.start_with_config(config, &annotations)?;
    
    Ok(())
}
```
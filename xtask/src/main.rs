use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::path::{Path, PathBuf};
use xshell::{cmd, Shell};

#[derive(Parser)]
#[command(author, version, about = "Development tasks for crashpad-rs")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Build the project
    Build {
        /// Build in release mode
        #[arg(long)]
        release: bool,
    },
    /// Create a distribution package
    Dist {
        /// Target directory for distribution
        #[arg(long, default_value = "dist")]
        output: PathBuf,
    },
    /// Run tests
    Test,
    /// Clean build artifacts
    Clean,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let sh = Shell::new()?;

    match cli.command {
        Commands::Build { release } => build(&sh, release)?,
        Commands::Dist { output } => dist(&sh, &output)?,
        Commands::Test => test(&sh)?,
        Commands::Clean => clean(&sh)?,
    }

    Ok(())
}

fn build(sh: &Shell, release: bool) -> Result<()> {
    println!("Building crashpad-rs...");

    if release {
        cmd!(sh, "cargo build --release")
            .run()
            .context("Failed to build")?;
    } else {
        cmd!(sh, "cargo build").run().context("Failed to build")?;
    }

    println!("Build complete!");
    Ok(())
}

fn dist(sh: &Shell, output_dir: &Path) -> Result<()> {
    println!("Creating distribution package...");

    // Build in release mode first
    build(sh, true)?;

    // Create output directory structure
    sh.create_dir(output_dir)?;
    let lib_dir = output_dir.join("lib");
    let include_dir = output_dir.join("include");
    let bin_dir = output_dir.join("bin");

    sh.create_dir(&lib_dir)?;
    sh.create_dir(&include_dir)?;
    sh.create_dir(&bin_dir)?;

    // Find the workspace root
    let workspace_root = find_workspace_root(sh)?;

    // Detect platform
    let (os, arch) = detect_platform();
    let platform = format!("{os}-{arch}");

    // Handler executable name
    let handler_name = if cfg!(windows) {
        "crashpad_handler.exe"
    } else {
        "crashpad_handler"
    };

    // Find and copy the built handler
    let handler_path = workspace_root
        .join("third_party/crashpad_checkout/crashpad/out")
        .join(&platform)
        .join(handler_name);

    if !handler_path.exists() {
        anyhow::bail!(
            "crashpad_handler not found at: {}\nMake sure to build crashpad-sys first",
            handler_path.display()
        );
    }

    let dist_handler = bin_dir.join(handler_name);
    sh.copy_file(&handler_path, &dist_handler)?;
    println!("✓ Copied crashpad_handler to dist/bin/");

    // Copy Rust libraries
    let target_release = workspace_root.join("target/release");

    // Copy .rlib files (Rust libraries)
    for entry in std::fs::read_dir(&target_release)? {
        let entry = entry?;
        let path = entry.path();
        if let Some(name) = path.file_name() {
            let name_str = name.to_string_lossy();
            if name_str.starts_with("libcrashpad")
                && (name_str.ends_with(".rlib") || name_str.ends_with(".a"))
            {
                let dest = lib_dir.join(name);
                sh.copy_file(&path, &dest)?;
                println!("✓ Copied library: {name_str}");
            }
        }
    }

    // Copy header files
    let crashpad_sys_dir = workspace_root.join("crashpad-sys");
    if crashpad_sys_dir.join("wrapper.h").exists() {
        sh.copy_file(
            crashpad_sys_dir.join("wrapper.h"),
            include_dir.join("crashpad_wrapper.h"),
        )?;
        println!("✓ Copied header: crashpad_wrapper.h");
    }

    // Create README for distribution
    let readme_content = format!(
        r#"# Crashpad-rs Distribution Package

Platform: {platform}

## Directory Structure

```
dist/
├── lib/          # Rust libraries (.rlib, .a)
├── include/      # C/C++ header files
├── bin/          # Executables (crashpad_handler)
└── README.md     # This file
```

## Contents

- `bin/{handler_name}` - The Crashpad handler executable
- `lib/libcrashpad*.rlib` - Rust library files
- `include/crashpad_wrapper.h` - C API header

## Integration

### For Rust Projects

Add to your `Cargo.toml`:
```toml
[dependencies]
crashpad = {{ path = "path/to/dist/lib" }}
```

### Usage Example

```rust
use crashpad::{{CrashpadClient, CrashpadConfig}};
use std::collections::HashMap;

let client = CrashpadClient::new()?;

let config = CrashpadConfig::builder()
    .handler_path("/path/to/crashpad_handler")  // Or leave empty for auto-detection
    .database_path("./crashes")
    .build();

let mut annotations = HashMap::new();
annotations.insert("version".to_string(), "1.0.0".to_string());

client.start_with_config(&config, &annotations)?;
```

## Deployment

When deploying your application:
1. Copy `bin/{handler_name}` to the same directory as your executable
2. Or install it system-wide in `/usr/local/bin` (Unix) or Program Files (Windows)
3. Or set `CRASHPAD_HANDLER` environment variable to its location
"#
    );

    sh.write_file(output_dir.join("README.md"), readme_content)?;

    // Create a simple Cargo.toml for the distribution
    let cargo_toml = r#"[package]
name = "crashpad-dist"
version = "0.1.0"
edition = "2021"

[lib]
path = "lib/libcrashpad.rlib"

[dependencies]
crashpad-sys = { path = "lib" }
"#
    .to_string();
    sh.write_file(output_dir.join("Cargo.toml"), cargo_toml)?;

    println!(
        "\n✓ Distribution package created at: {}",
        output_dir.display()
    );
    println!("  Platform: {platform}");
    println!("\nDirectory structure:");
    println!("  lib/      - Rust libraries");
    println!("  include/  - Header files");
    println!("  bin/      - crashpad_handler executable");
    println!("  examples/ - Example applications");

    Ok(())
}

fn test(sh: &Shell) -> Result<()> {
    println!("Running tests...");
    cmd!(sh, "cargo test").run()?;
    Ok(())
}

fn clean(sh: &Shell) -> Result<()> {
    println!("Cleaning build artifacts...");
    cmd!(sh, "cargo clean").run()?;

    // Also clean distribution directory
    let dist_dir = PathBuf::from("dist");
    if dist_dir.exists() {
        sh.remove_path(&dist_dir)?;
        println!("✓ Removed dist/");
    }

    Ok(())
}

fn find_workspace_root(sh: &Shell) -> Result<PathBuf> {
    let output = cmd!(sh, "cargo metadata --no-deps --format-version 1")
        .read()
        .context("Failed to get cargo metadata")?;

    let metadata: serde_json::Value = serde_json::from_str(&output)?;
    let workspace_root = metadata["workspace_root"]
        .as_str()
        .context("Failed to find workspace root")?;

    Ok(PathBuf::from(workspace_root))
}

fn detect_platform() -> (&'static str, &'static str) {
    let os = if cfg!(target_os = "macos") {
        "macos"
    } else if cfg!(target_os = "linux") {
        "linux"
    } else if cfg!(target_os = "windows") {
        "windows"
    } else {
        "unknown"
    };

    let arch = if cfg!(target_arch = "x86_64") {
        "x86_64"
    } else if cfg!(target_arch = "aarch64") {
        "aarch64"
    } else {
        "unknown"
    };

    (os, arch)
}


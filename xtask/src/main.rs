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
    /// Install external development tools
    InstallTools,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let sh = Shell::new()?;

    match cli.command {
        Commands::Build { release } => build(&sh, release)?,
        Commands::Dist { output } => dist(&sh, &output)?,
        Commands::Test => test(&sh)?,
        Commands::Clean => clean(&sh)?,
        Commands::InstallTools => install_tools(&sh)?,
    }

    Ok(())
}

fn build(sh: &Shell, release: bool) -> Result<()> {
    println!("Building crashpad-rs...");

    let mode = if release { "--release" } else { "" };
    cmd!(sh, "cargo build {mode}").run()?;

    println!("✅ Build completed successfully!");
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
    let target_dir = workspace_root.join("target/release");
    let lib_files = ["libcrashpad.rlib", "libcrashpad_sys.rlib"];

    for lib in &lib_files {
        let src = target_dir.join(lib);
        if src.exists() {
            let dest = lib_dir.join(lib);
            sh.copy_file(&src, &dest)?;
            println!("✓ Copied {lib} to dist/lib/");
        }
    }

    // Copy include files
    let sys_dir = workspace_root.join("crashpad-sys");
    let wrapper_h = sys_dir.join("wrapper.h");
    if wrapper_h.exists() {
        sh.copy_file(&wrapper_h, include_dir.join("wrapper.h"))?;
        println!("✓ Copied wrapper.h to dist/include/");
    }

    // Create README for the distribution
    let readme_content = format!(
        r#"# Crashpad Distribution Package

Platform: {platform}
Build: Release

## Contents

- `bin/` - Crashpad handler executable
- `lib/` - Rust libraries
- `include/` - Header files

## Usage

1. Set the handler path in your code:
   ```rust
   use crashpad::CrashpadConfig;

   let config = CrashpadConfig::new()
       .database_path("./crashes")
       .handler_path("./dist/bin/{handler_name}")
       .build();
   ```

2. Link the libraries in your Cargo.toml or build script.

For more information, see the main README.md in the repository.
"#
    );

    sh.write_file(output_dir.join("README.md"), readme_content)?;
    println!("✓ Created README.md");

    println!(
        "\n✅ Distribution package created at: {}",
        output_dir.display()
    );
    Ok(())
}

fn test(sh: &Shell) -> Result<()> {
    println!("Running tests...");

    // Run unit tests
    cmd!(sh, "cargo test --lib").run()?;

    // Run integration tests with nextest for process isolation
    cmd!(sh, "cargo nextest run --test '*'").run()?;

    println!("✅ All tests passed!");
    Ok(())
}

fn clean(sh: &Shell) -> Result<()> {
    println!("Cleaning build artifacts...");

    // Clean Rust target
    cmd!(sh, "cargo clean").run()?;

    // Clean native build artifacts
    let workspace_root = find_workspace_root(sh)?;
    let native_dirs = ["third_party/crashpad_checkout", "third_party/depot_tools"];

    for dir in &native_dirs {
        let path = workspace_root.join(dir);
        if path.exists() {
            sh.remove_path(&path)?;
            println!("✓ Removed {dir}");
        }
    }

    println!("✅ Clean completed!");
    Ok(())
}

fn install_tools(sh: &Shell) -> Result<()> {
    println!("Installing development tools...");

    // Install cargo-nextest for better test isolation
    println!("Installing cargo-nextest...");
    cmd!(sh, "cargo install cargo-nextest --locked").run()?;

    // Install cargo-ndk for Android cross-compilation
    println!("Installing cargo-ndk...");
    cmd!(sh, "cargo install cargo-ndk").run()?;

    println!("✅ Tools installed successfully!");
    Ok(())
}

fn find_workspace_root(sh: &Shell) -> Result<PathBuf> {
    let output = cmd!(sh, "cargo metadata --no-deps --format-version 1")
        .read()
        .context("Failed to get cargo metadata")?;

    let metadata: serde_json::Value = serde_json::from_str(&output)?;
    let root = metadata["workspace_root"]
        .as_str()
        .context("Failed to get workspace root")?;

    Ok(PathBuf::from(root))
}

fn detect_platform() -> (&'static str, &'static str) {
    let os = if cfg!(target_os = "macos") {
        "mac"
    } else if cfg!(target_os = "linux") {
        "linux"
    } else if cfg!(target_os = "windows") {
        "win"
    } else {
        "unknown"
    };

    let arch = if cfg!(target_arch = "x86_64") {
        "x64"
    } else if cfg!(target_arch = "aarch64") {
        "arm64"
    } else if cfg!(target_arch = "x86") {
        "x86"
    } else {
        "unknown"
    };

    (os, arch)
}

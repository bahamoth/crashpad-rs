/// Build script for the crashpad-sys crate.
///
/// This script orchestrates the entire Crashpad build process.
#[path = "build/config.rs"]
mod config;
#[path = "build/phases.rs"]
mod phases;
#[path = "build/tools.rs"]
mod tools;

use config::BuildConfig;
use phases::BuildPhases;

fn main() {
    if let Err(e) = run() {
        eprintln!("Build failed: {e}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    // Load platform configuration
    let config = BuildConfig::from_env()?;

    // Check if we're in cargo package environment
    // cargo package doesn't need actual build, just verification
    if config
        .manifest_dir
        .to_string_lossy()
        .contains("target/package/")
    {
        eprintln!("Detected cargo package environment, skipping Crashpad build");
        // Create dummy bindings file for package verification
        let bindings_path = config.bindings_path();
        if let Some(parent) = bindings_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(bindings_path, "// Placeholder for cargo package\n")?;
        return Ok(());
    }

    let mut phases = BuildPhases::new(config);

    // Set up cargo rebuild triggers
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=wrapper.h");
    println!("cargo:rerun-if-changed=crashpad_wrapper.cc");

    // Execute all build phases in order
    phases.prepare()?; // Phase 1: prepare build tools
    phases.configure()?; // Phase 2: GN configuration
    phases.build()?; // Phase 3: Ninja build
    phases.wrapper()?; // Phase 4: Wrapper compilation
    phases.package()?; // Phase 5: Static library creation
    phases.bindgen()?; // Phase 6: FFI bindings generation
    phases.emit_link()?; // Phase 7: Cargo link metadata

    Ok(())
}

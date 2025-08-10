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

/// Build script for the crashpad-sys crate.
///
/// This script assembles the build phases in order.
/// For pre-built packages, phases 1-3 are handled by `cargo xtask prebuild`.
/// This script runs phases 4-7 for final assembly.
use crashpad_build::{BuildConfig, BuildPhases};

fn main() {
    if let Err(e) = run() {
        eprintln!("Build failed: {e}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    // Load platform configuration
    let config = BuildConfig::from_env()?;
    let phases = BuildPhases::new(config);

    // Set up cargo rebuild triggers
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=wrapper.h");
    println!("cargo:rerun-if-changed=crashpad_wrapper.cc");

    // Execute all build phases in order
    // For pre-built usage, phases 1-3 should be skipped or no-op
    phases.prepare()?; // Phase 1: depot_tools, crashpad source
    phases.configure()?; // Phase 2: GN configuration
    phases.build()?; // Phase 3: Ninja build
    phases.wrapper()?; // Phase 4: Wrapper compilation
    phases.package()?; // Phase 5: Static library creation
    phases.bindgen()?; // Phase 6: FFI bindings generation
    phases.emit_link()?; // Phase 7: Cargo link metadata

    Ok(())
}

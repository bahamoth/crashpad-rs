/// Build script for the crashpad-sys crate.
///
/// This script handles the compilation of Google Crashpad and generates Rust bindings.
/// It manages all build dependencies including depot_tools and uses the Chromium build system.
///
/// The build process follows a simple phase-based approach:
/// - Direct orchestration without unnecessary abstraction layers
/// - Platform configuration centralized in one place
/// - Clear separation between build phases
mod build {
    pub mod config;
    pub mod phases;
}

use build::config::BuildConfig;
use build::phases::BuildPhases;

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
    println!("cargo:rerun-if-changed=build/config.rs");
    println!("cargo:rerun-if-changed=build/phases.rs");

    // Execute build phases in order
    phases.prepare()?; // depot_tools, crashpad source
    phases.configure()?; // GN configuration
    phases.build()?; // Ninja build
    phases.wrapper()?; // Wrapper compilation
    phases.package()?; // Static library creation
    phases.bindgen()?; // FFI bindings generation
    phases.emit_link()?; // Cargo link metadata

    Ok(())
}

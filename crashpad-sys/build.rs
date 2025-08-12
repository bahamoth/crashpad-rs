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
    // Check if we're building on docs.rs
    if std::env::var("DOCS_RS").is_ok() {
        println!("cargo:warning=docs.rs build detected, skipping native build");

        // Create dummy bindings for docs.rs
        let out_dir = std::env::var("OUT_DIR").unwrap();
        let bindings_path = std::path::Path::new(&out_dir).join("bindings.rs");

        // Create minimal bindings to allow documentation build
        // These match the actual C API in wrapper.h
        std::fs::write(
            &bindings_path,
            r#"
            //! Dummy bindings for docs.rs build
            //! 
            //! These are placeholder types to allow documentation generation.
            //! Real bindings are generated during normal builds.
            
            use std::os::raw::{c_char, c_void};
            
            // Opaque handle types
            pub type crashpad_client_t = *mut c_void;
            
            // Core functions from wrapper.h
            extern "C" {
                pub fn crashpad_client_new() -> crashpad_client_t;
                pub fn crashpad_client_delete(client: crashpad_client_t);
                pub fn crashpad_client_start_handler(
                    client: crashpad_client_t,
                    handler_path: *const c_char,
                    database_path: *const c_char,
                    metrics_path: *const c_char,
                    url: *const c_char,
                    annotations_keys: *const *const c_char,
                    annotations_values: *const *const c_char,
                    annotations_count: usize,
                ) -> bool;
            }
        "#,
        )
        .expect("Failed to write dummy bindings");

        return;
    }

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

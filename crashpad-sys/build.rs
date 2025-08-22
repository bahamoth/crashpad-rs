/// Build script for the crashpad-sys crate.
///
/// This script orchestrates the entire Crashpad build process.
#[path = "build/cache.rs"]
mod cache;
#[path = "build/config.rs"]
mod config;
#[path = "build/depot_build.rs"]
#[cfg(any(
    feature = "vendored-depot",
    not(any(feature = "vendored", feature = "vendored-depot", feature = "prebuilt"))
))]
mod depot_build;
#[path = "build/phases.rs"]
mod phases;
#[path = "build/prebuilt.rs"]
#[cfg(feature = "prebuilt")]
mod prebuilt;
#[path = "build/tools.rs"]
mod tools;

#[cfg(any(
    feature = "vendored",
    not(any(feature = "vendored", feature = "vendored-depot", feature = "prebuilt"))
))]
use config::BuildConfig;
#[cfg(any(
    feature = "vendored",
    not(any(feature = "vendored", feature = "vendored-depot", feature = "prebuilt"))
))]
use phases::BuildPhases;

fn main() {
    // Feature flag validation - ensure only one build strategy is selected
    #[cfg(all(feature = "vendored", feature = "vendored-depot"))]
    compile_error!(
        "Only one build strategy can be selected: vendored, vendored-depot, or prebuilt"
    );

    #[cfg(all(feature = "vendored", feature = "prebuilt"))]
    compile_error!(
        "Only one build strategy can be selected: vendored, vendored-depot, or prebuilt"
    );

    #[cfg(all(feature = "vendored-depot", feature = "prebuilt"))]
    compile_error!(
        "Only one build strategy can be selected: vendored, vendored-depot, or prebuilt"
    );

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
            // Dummy bindings for docs.rs build
            // 
            // These are placeholder types to allow documentation generation.
            // Real bindings are generated during normal builds.
            
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

    // Dispatch based on build strategy
    #[cfg(feature = "prebuilt")]
    {
        println!("cargo:warning=Using prebuilt strategy");
        if let Err(e) = prebuilt::download_and_link() {
            eprintln!("Prebuilt download failed: {e}");
            std::process::exit(1);
        }
        return;
    }

    #[cfg(all(not(feature = "prebuilt"), feature = "vendored-depot"))]
    {
        println!("cargo:warning=Using vendored-depot strategy");
        println!("cargo:warning=[BUILD.RS] Starting depot_build::build_with_depot_tools()");
        match depot_build::build_with_depot_tools() {
            Ok(_) => {
                println!("cargo:warning=[BUILD.RS] depot_build completed successfully");
            }
            Err(e) => {
                println!("cargo:warning=[BUILD.RS] depot_tools build failed: {}", e);
                println!("cargo:warning=[BUILD.RS] Error details: {:?}", e);
                std::process::exit(1);
            }
        }
        return;
    }

    #[cfg(all(
        not(feature = "prebuilt"),
        not(feature = "vendored-depot"),
        feature = "vendored"
    ))]
    {
        println!("cargo:warning=Using vendored strategy");
        if let Err(e) = run() {
            eprintln!("Build failed: {e}");
            std::process::exit(1);
        }
    }

    // No feature selected - auto-select based on platform
    #[cfg(not(any(feature = "vendored", feature = "vendored-depot", feature = "prebuilt")))]
    {
        println!("cargo:warning=No build strategy specified, auto-selecting based on platform");

        let target = std::env::var("TARGET").unwrap_or_default();

        if target.contains("windows") {
            // Windows requires depot_tools for proper build
            println!("cargo:warning=Auto-selected vendored-depot strategy for Windows");
            if let Err(e) = depot_build::build_with_depot_tools() {
                eprintln!("depot_tools build failed: {e}");
                std::process::exit(1);
            }
        } else {
            // Linux/macOS/iOS/Android can all use vendored (standalone tools)
            println!(
                "cargo:warning=Auto-selected vendored strategy for {}",
                target
            );
            if let Err(e) = run() {
                eprintln!("Build failed: {e}");
                std::process::exit(1);
            }
        }
    }
}

#[cfg(any(
    feature = "vendored",
    not(any(feature = "vendored", feature = "vendored-depot", feature = "prebuilt"))
))]
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
    phases
        .prepare()
        .map_err(|e| format!("Phase 1 (prepare) failed: {e}"))?;
    phases
        .configure()
        .map_err(|e| format!("Phase 2 (configure) failed: {e}"))?;
    phases
        .build()
        .map_err(|e| format!("Phase 3 (build) failed: {e}"))?;
    phases
        .wrapper()
        .map_err(|e| format!("Phase 4 (wrapper) failed: {e}"))?;
    phases
        .package()
        .map_err(|e| format!("Phase 5 (package) failed: {e}"))?;
    phases
        .bindgen()
        .map_err(|e| format!("Phase 6 (bindgen) failed: {e}"))?;
    phases
        .emit_link()
        .map_err(|e| format!("Phase 7 (emit_link) failed: {e}"))?;

    Ok(())
}

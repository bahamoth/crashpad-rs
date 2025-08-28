use std::env;

/// Build script for the crashpad crate.
///
/// This script only validates feature flags to ensure proper configuration.
fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=DEP_CRASHPAD_HANDLER");

    // Check feature flags
    let bundled = env::var("CARGO_FEATURE_BUNDLED").is_ok();
    let system = env::var("CARGO_FEATURE_SYSTEM").is_ok();

    if bundled && system {
        panic!("Cannot enable both 'bundled' and 'system' features");
    }

    // Pass-through handler path from crashpad-rs-sys to dependents of `crashpad`.
    // crashpad-rs-sys uses links = "crashpad" and prints cargo:handler=...
    // We re-expose it so top-level crates that depend only on `crashpad` can access it
    // as DEP_CRASHPAD_RS_HANDLER.
    if let Ok(handler) = env::var("DEP_CRASHPAD_HANDLER") {
        println!("cargo:handler={}", handler);
    }
}

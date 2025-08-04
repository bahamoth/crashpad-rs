use std::env;

/// Build script for the crashpad crate.
/// 
/// This script only validates feature flags to ensure proper configuration.
fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    
    // Check feature flags
    let bundled = env::var("CARGO_FEATURE_BUNDLED").is_ok();
    let system = env::var("CARGO_FEATURE_SYSTEM").is_ok();
    
    if bundled && system {
        panic!("Cannot enable both 'bundled' and 'system' features");
    }
}
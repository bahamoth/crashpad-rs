#[path = "build/mod.rs"]
mod build;

/// Build script for the crashpad-sys crate.
///
/// This script handles the compilation of Google Crashpad and generates Rust bindings.
/// It manages all build dependencies including depot_tools and uses the Chromium build system.
///
/// The build process is now modularized following ARCHITECTURE.md principles:
/// - Platform differences handled by enum-based strategy pattern
/// - Clear separation of concerns
/// - Testable, maintainable code structure
fn main() {
    if let Err(e) = build::CrashpadBuilder::new().and_then(|builder| builder.build()) {
        eprintln!("Build failed: {}", e);
        std::process::exit(1);
    }
}
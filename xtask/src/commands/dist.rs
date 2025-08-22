use anyhow::Result;
use xshell::{cmd, Shell};

use crate::commands::symlink::create_symlinks;

pub fn dist(sh: &Shell) -> Result<()> {
    println!("Packaging crates for distribution...");

    // First create symlinks/copies for packaging
    println!("\n📦 Preparing dependencies...");
    create_symlinks(sh)?;

    // Package crashpad-rs-sys
    println!("\n📦 Packaging crashpad-rs-sys...");
    cmd!(sh, "cargo package --package crashpad-rs-sys").run()?;
    println!("✓ crashpad-rs-sys packaged successfully");

    // Package crashpad
    println!("\n📦 Packaging crashpad...");
    cmd!(sh, "cargo package --package crashpad").run()?;
    println!("✓ crashpad packaged successfully");

    println!("\n✅ Packages created in target/package/");
    println!("📤 Ready to publish to crates.io with 'cargo publish'");
    Ok(())
}

use anyhow::Result;
use xshell::{cmd, Shell};

pub fn install_tools(sh: &Shell) -> Result<()> {
    println!("Installing development tools...");

    // Install cargo-nextest for better test isolation
    println!("Installing cargo-nextest...");
    cmd!(sh, "cargo install cargo-nextest --locked").run()?;

    // Install cargo-ndk for Android cross-compilation
    println!("Installing cargo-ndk...");
    cmd!(sh, "cargo install cargo-ndk").run()?;

    println!("✅ Tools installed successfully!");
    Ok(())
}
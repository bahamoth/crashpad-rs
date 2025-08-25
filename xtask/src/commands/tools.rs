use anyhow::Result;
use xshell::{cmd, Shell};

pub fn install_tools(sh: &Shell) -> Result<()> {
    println!("Installing development tools...");

    // Install just - cross-platform command runner (make replacement)
    println!("Installing just...");
    cmd!(sh, "cargo install just --locked").run()?;

    // Install cargo-nextest for better test isolation
    println!("Installing cargo-nextest...");
    cmd!(sh, "cargo install cargo-nextest --locked").run()?;

    // Install cargo-ndk for Android cross-compilation
    println!("Installing cargo-ndk...");
    cmd!(sh, "cargo install cargo-ndk").run()?;

    println!("✅ Tools installed successfully!");
    println!("📝 You can now use 'just' commands instead of 'make'");
    Ok(())
}

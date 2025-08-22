use anyhow::Result;
use xshell::{cmd, Shell};

pub fn build(sh: &Shell, release: bool) -> Result<()> {
    println!("Building crashpad-rs...");

    if release {
        cmd!(sh, "cargo build --release").run()?;
    } else {
        cmd!(sh, "cargo build").run()?;
    }

    println!("✅ Build completed successfully!");
    Ok(())
}

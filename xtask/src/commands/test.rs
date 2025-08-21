use anyhow::Result;
use xshell::{cmd, Shell};

pub fn test(sh: &Shell) -> Result<()> {
    println!("Running tests...");

    // Run unit tests
    cmd!(sh, "cargo test --lib").run()?;

    // Run integration tests with nextest for process isolation
    cmd!(sh, "cargo nextest run --test '*'").run()?;

    println!("✅ All tests passed!");
    Ok(())
}
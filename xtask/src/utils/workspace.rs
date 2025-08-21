use anyhow::{Context, Result};
use std::path::PathBuf;
use xshell::{cmd, Shell};

/// Find the workspace root directory using cargo metadata
pub fn find_workspace_root(sh: &Shell) -> Result<PathBuf> {
    let output = cmd!(sh, "cargo metadata --no-deps --format-version 1")
        .read()
        .context("Failed to get cargo metadata")?;

    let metadata: serde_json::Value = serde_json::from_str(&output)?;
    let root = metadata["workspace_root"]
        .as_str()
        .context("Failed to get workspace root")?;

    Ok(PathBuf::from(root))
}

/// Find the OUT_DIR for a specific target
#[allow(dead_code)]
pub fn find_out_dir(sh: &Shell, target: &str) -> Result<PathBuf> {
    let workspace_root = find_workspace_root(sh)?;

    // Common OUT_DIR patterns
    let candidates = vec![
        workspace_root.join(format!("target/{}/debug/build", target)),
        workspace_root.join(format!("target/{}/release/build", target)),
        workspace_root.join("target/debug/build"),
        workspace_root.join("target/release/build"),
    ];

    for candidate in candidates {
        if candidate.exists() {
            // Find crashpad-rs-sys build directory
            for entry in std::fs::read_dir(&candidate)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() && path.to_string_lossy().contains("crashpad-rs-sys") {
                    let out_dir = path.join("out");
                    if out_dir.exists() {
                        return Ok(out_dir);
                    }
                }
            }
        }
    }

    anyhow::bail!(
        "Could not find OUT_DIR for target {}. Make sure the build completed successfully.",
        target
    )
}
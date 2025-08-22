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

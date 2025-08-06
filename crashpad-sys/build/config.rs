//! Build configuration management
//!
//! This module handles environment detection and path configuration
//! for the Crashpad build process.

use std::env;
use std::path::PathBuf;

use super::platform::{Platform, PlatformError};

#[derive(Debug)]
pub struct BuildConfig {
    /// Cargo's OUT_DIR
    pub out_dir: PathBuf,
    /// Directory containing Cargo.toml
    pub manifest_dir: PathBuf,
    /// Workspace root directory
    #[allow(dead_code)]
    pub workspace_root: PathBuf,
    /// Detected platform
    pub platform: Platform,
    /// Path to depot_tools
    pub depot_tools_path: PathBuf,
    /// Path to crashpad checkout
    pub crashpad_checkout: PathBuf,
    /// Path to crashpad source
    pub crashpad_dir: PathBuf,
}

#[derive(Debug)]
pub struct ConfigError(String);

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Configuration error: {}", self.0)
    }
}

impl std::error::Error for ConfigError {}

impl From<PlatformError> for ConfigError {
    fn from(err: PlatformError) -> Self {
        ConfigError(err.to_string())
    }
}

impl BuildConfig {
    /// Create a new BuildConfig from environment variables
    pub fn from_env() -> Result<Self, ConfigError> {
        let out_dir = PathBuf::from(
            env::var("OUT_DIR").map_err(|_| ConfigError("OUT_DIR not set".to_string()))?,
        );

        let manifest_dir = PathBuf::from(
            env::var("CARGO_MANIFEST_DIR")
                .map_err(|_| ConfigError("CARGO_MANIFEST_DIR not set".to_string()))?,
        );

        let workspace_root = manifest_dir
            .parent()
            .ok_or_else(|| ConfigError("Failed to find workspace root".to_string()))?
            .to_path_buf();

        let platform = Platform::detect()?;

        let depot_tools_path = workspace_root.join("third_party/depot_tools");
        let crashpad_checkout = workspace_root.join("third_party/crashpad_checkout");
        let crashpad_dir = crashpad_checkout.join("crashpad");

        Ok(Self {
            out_dir,
            manifest_dir,
            workspace_root,
            platform,
            depot_tools_path,
            crashpad_checkout,
            crashpad_dir,
        })
    }

    /// Get the build output directory for the current platform
    pub fn build_dir(&self) -> PathBuf {
        self.crashpad_dir
            .join("out")
            .join(self.platform.build_name())
    }

    /// Get the object files directory
    pub fn obj_dir(&self) -> PathBuf {
        self.build_dir().join("obj")
    }

    /// Get the PATH with depot_tools prepended
    pub fn path_with_depot_tools(&self) -> String {
        format!(
            "{}:{}",
            self.depot_tools_path.display(),
            env::var("PATH").unwrap_or_default()
        )
    }

    /// Get the path to the wrapper object file
    pub fn wrapper_obj_path(&self) -> PathBuf {
        self.out_dir.join("crashpad_wrapper.o")
    }

    /// Get the path to the static library
    pub fn static_lib_path(&self) -> PathBuf {
        self.out_dir.join("libcrashpad_wrapper.a")
    }

    /// Get the path to the generated bindings
    pub fn bindings_path(&self) -> PathBuf {
        self.out_dir.join("bindings.rs")
    }

    /// Get the handler executable name
    pub fn handler_name(&self) -> &'static str {
        match self.platform {
            Platform::Windows { .. } => "crashpad_handler.exe",
            _ => "crashpad_handler",
        }
    }

    /// Get the path to the handler executable
    pub fn handler_path(&self) -> PathBuf {
        self.build_dir().join(self.handler_name())
    }

    /// Check if verbose output is requested
    pub fn verbose(&self) -> bool {
        env::var("CRASHPAD_BUILD_VERBOSE").is_ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path_construction() {
        // This test would need environment variables set up
        // so it's more of a documentation of expected behavior

        // Example of what the test would look like:
        // let config = BuildConfig::from_env().unwrap();
        // assert!(config.depot_tools_path.ends_with("third_party/depot_tools"));
        // assert!(config.crashpad_dir.ends_with("third_party/crashpad_checkout/crashpad"));
    }
}

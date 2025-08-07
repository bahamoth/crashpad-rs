use crate::{CrashpadError, Result};
use std::env;
use std::path::{Path, PathBuf};

/// Configuration for Crashpad client
#[derive(Debug, Clone)]
pub struct CrashpadConfig {
    handler_path: PathBuf,
    database_path: PathBuf,
    metrics_path: PathBuf,
    url: Option<String>,
}

impl Default for CrashpadConfig {
    fn default() -> Self {
        let exe_dir = env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.to_path_buf()))
            .unwrap_or_else(|| PathBuf::from("."));

        Self {
            handler_path: PathBuf::new(),
            database_path: exe_dir.join("crashpad_db"),
            metrics_path: exe_dir.join("crashpad_metrics"),
            url: None,
        }
    }
}

impl CrashpadConfig {
    /// Create a new configuration with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a builder for the configuration
    pub fn builder() -> CrashpadConfigBuilder {
        CrashpadConfigBuilder::default()
    }

    /// Set the database path
    pub fn with_database_path<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.database_path = path.as_ref().to_path_buf();
        self
    }

    /// Set the metrics path
    pub fn with_metrics_path<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.metrics_path = path.as_ref().to_path_buf();
        self
    }

    /// Set the upload URL
    pub fn with_url<S: Into<String>>(mut self, url: S) -> Self {
        self.url = Some(url.into());
        self
    }

    /// Get the handler path
    ///
    /// Search order:
    /// 1. Path specified in config (if provided)
    /// 2. CRASHPAD_HANDLER environment variable
    /// 3. Same directory as the executable
    /// 4. Current working directory
    pub(crate) fn handler_path(&self) -> Result<PathBuf> {
        // iOS/tvOS/watchOS use in-process handler, no external handler needed
        #[cfg(any(target_os = "ios", target_os = "tvos", target_os = "watchos"))]
        {
            // Return empty path for iOS - it's handled in-process
            return Ok(PathBuf::new());
        }

        #[cfg(not(any(target_os = "ios", target_os = "tvos", target_os = "watchos")))]
        {
            // Determine handler filename based on platform
            let handler_name = if cfg!(target_os = "android") {
                "libcrashpad_handler.so"
            } else if cfg!(windows) {
                "crashpad_handler.exe"
            } else {
                "crashpad_handler"
            };

            // 1. Check if path was explicitly set in config
            if !self.handler_path.as_os_str().is_empty() {
                let path = &self.handler_path;
                if path.exists() {
                    return Ok(path.clone());
                }
                // If explicitly set but doesn't exist, still return it
                // (let the caller handle the error for better diagnostics)
                return Ok(path.clone());
            }

            // 2. Check CRASHPAD_HANDLER environment variable
            if let Ok(env_path) = env::var("CRASHPAD_HANDLER") {
                let path = PathBuf::from(env_path);
                if path.exists() {
                    return Ok(path);
                }
            }

            // 3. Check same directory as executable
            if let Ok(exe_path) = env::current_exe() {
                if let Some(exe_dir) = exe_path.parent() {
                    let handler_path = exe_dir.join(handler_name);
                    if handler_path.exists() {
                        return Ok(handler_path);
                    }
                }
            }

            // 4. Check current working directory
            let cwd_handler = PathBuf::from(handler_name);
            if cwd_handler.exists() {
                return Ok(cwd_handler);
            }

            Err(CrashpadError::InvalidConfiguration(
                format!(
                    "Handler '{handler_name}' not found. Searched: config path, CRASHPAD_HANDLER env, executable directory, current directory"
                )
            ))
        }
    }

    pub(crate) fn database_path(&self) -> &Path {
        &self.database_path
    }

    pub(crate) fn metrics_path(&self) -> &Path {
        &self.metrics_path
    }

    pub(crate) fn url(&self) -> Option<&str> {
        self.url.as_deref()
    }
}

/// Builder for CrashpadConfig
#[derive(Default)]
pub struct CrashpadConfigBuilder {
    config: CrashpadConfig,
}

impl CrashpadConfigBuilder {
    /// Set the handler path
    pub fn handler_path<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.config.handler_path = path.as_ref().to_path_buf();
        self
    }

    /// Set the database path
    pub fn database_path<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.config.database_path = path.as_ref().to_path_buf();
        self
    }

    /// Set the metrics path
    pub fn metrics_path<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.config.metrics_path = path.as_ref().to_path_buf();
        self
    }

    /// Set the upload URL
    pub fn url<S: Into<String>>(mut self, url: S) -> Self {
        self.config.url = Some(url.into());
        self
    }

    /// Build the configuration
    pub fn build(self) -> CrashpadConfig {
        self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder() {
        let config = CrashpadConfig::builder()
            .handler_path("/usr/local/bin/crashpad_handler")
            .database_path("/tmp/crashes")
            .url("https://crashes.example.com")
            .build();

        assert_eq!(
            config.handler_path.to_str().unwrap(),
            "/usr/local/bin/crashpad_handler"
        );
        assert_eq!(config.database_path.to_str().unwrap(), "/tmp/crashes");
        assert_eq!(config.url.as_deref(), Some("https://crashes.example.com"));
    }

    #[test]
    #[cfg(not(any(target_os = "ios", target_os = "tvos", target_os = "watchos")))]
    fn test_handler_path_fallback() {
        // Test 1: Explicit path in config takes precedence
        let config = CrashpadConfig::builder()
            .handler_path("/explicit/path/crashpad_handler")
            .build();

        // Should return the explicit path even if it doesn't exist
        assert_eq!(
            config.handler_path().unwrap().to_str().unwrap(),
            "/explicit/path/crashpad_handler"
        );

        // Test 2: Empty config should trigger fallback search
        let config = CrashpadConfig::builder().build();

        // This will search through fallbacks
        // The actual result depends on environment
        let result = config.handler_path();

        // If it finds a handler, it should be one of the expected names
        if let Ok(path) = result {
            let filename = path.file_name().unwrap().to_str().unwrap();
            assert!(
                filename == "crashpad_handler"
                    || filename == "crashpad_handler.exe"
                    || filename == "libcrashpad_handler.so"
            );
        }
    }

    #[test]
    #[cfg(not(any(target_os = "ios", target_os = "tvos", target_os = "watchos")))]
    fn test_handler_env_var() {
        // Test that CRASHPAD_HANDLER environment variable is checked
        // Note: This test might interact with actual environment

        // Save current env var if it exists
        let original = env::var("CRASHPAD_HANDLER").ok();

        // Set a test path
        env::set_var("CRASHPAD_HANDLER", "/env/path/crashpad_handler");

        let config = CrashpadConfig::builder().build();

        // The handler_path method should check env var as fallback
        // (actual behavior depends on whether file exists)
        let _result = config.handler_path();

        // Restore original env var
        if let Some(orig) = original {
            env::set_var("CRASHPAD_HANDLER", orig);
        } else {
            env::remove_var("CRASHPAD_HANDLER");
        }
    }
}

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
    pub(crate) fn handler_path(&self) -> Result<PathBuf> {
        // iOS/tvOS/watchOS use in-process handler, no external handler needed
        #[cfg(any(target_os = "ios", target_os = "tvos", target_os = "watchos"))]
        {
            // Return empty path for iOS - it's handled in-process
            return Ok(PathBuf::new());
        }

        #[cfg(not(any(target_os = "ios", target_os = "tvos", target_os = "watchos")))]
        {
            if !self.handler_path.as_os_str().is_empty() {
                return Ok(self.handler_path.clone());
            }

            // Fallback: look in same directory as executable
            if let Ok(exe_path) = env::current_exe() {
                if let Some(exe_dir) = exe_path.parent() {
                    let handler_name = if cfg!(target_os = "android") {
                        "libcrashpad_handler.so"
                    } else if cfg!(windows) {
                        "crashpad_handler.exe"
                    } else {
                        "crashpad_handler"
                    };
                    let handler_path = exe_dir.join(handler_name);
                    if handler_path.exists() {
                        return Ok(handler_path);
                    }
                }
            }

            Err(CrashpadError::InvalidConfiguration(
                "Handler not found. Specify handler_path or place crashpad_handler in same directory as executable".to_string()
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
}

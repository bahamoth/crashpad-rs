/// Unified cache directory management for crashpad-rs
///
/// Simple module to provide consistent cache paths across all build methods
use std::env;
use std::path::PathBuf;

/// Get cache root directory
/// 
/// Priority:
/// 1. CRASHPAD_CACHE_DIR environment variable
/// 2. System cache directory + "crashpad-rs"
/// 3. Fallback to ".cache/crashpad-rs"
pub fn cache_root() -> PathBuf {
    env::var("CRASHPAD_CACHE_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            dirs::cache_dir()
                .unwrap_or_else(|| PathBuf::from(".cache"))
                .join("crashpad-rs")
        })
}

/// Get tools cache directory (for GN/Ninja binaries)
pub fn tools_dir() -> PathBuf {
    cache_root()
        .join("tools")
        .join(format!("{}-{}", env::consts::OS, env::consts::ARCH))
}

/// Get prebuilt cache directory
pub fn prebuilt_dir(version: &str, target: &str) -> PathBuf {
    cache_root()
        .join("prebuilt")
        .join(version)
        .join(target)
}
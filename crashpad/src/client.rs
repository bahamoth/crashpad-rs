use std::collections::HashMap;
use std::ffi::CString;
use std::path::Path;
use std::ptr;

use crate::{CrashpadConfig, CrashpadError, Result};

// Import FFI bindings
use crashpad_rs_sys::*;

/// A Crashpad client that can be used to capture and report crashes.
pub struct CrashpadClient {
    handle: crashpad_client_t,
}

impl CrashpadClient {
    /// Creates a new CrashpadClient instance.
    pub fn new() -> Result<Self> {
        let handle = unsafe { crashpad_client_new() };
        if handle.is_null() {
            return Err(CrashpadError::InitializationFailed);
        }
        Ok(CrashpadClient { handle })
    }

    /// Starts the Crashpad handler with a configuration.
    pub fn start_with_config(
        &self,
        config: &CrashpadConfig,
        annotations: &HashMap<String, String>,
    ) -> Result<()> {
        // iOS/tvOS/watchOS use in-process handler
        #[cfg(any(target_os = "ios", target_os = "tvos", target_os = "watchos"))]
        {
            // Get paths
            let database_path = config.database_path();
            let metrics_path = config.metrics_path();
            let url = config.url();

            // Ensure directories exist
            if let Some(parent) = database_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            if let Some(parent) = metrics_path.parent() {
                std::fs::create_dir_all(parent)?;
            }

            // For iOS, start in-process handler
            self.start_in_process_handler(database_path, metrics_path, url, annotations)
        }

        #[cfg(not(any(target_os = "ios", target_os = "tvos", target_os = "watchos")))]
        {
            // Get handler path (with fallback to same directory)
            let handler_path = config.handler_path()?;

            // Get paths
            let database_path = config.database_path();
            let metrics_path = config.metrics_path();
            let url = config.url();

            // Ensure directories exist
            if let Some(parent) = database_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            if let Some(parent) = metrics_path.parent() {
                std::fs::create_dir_all(parent)?;
            }

            self.start_handler(&handler_path, database_path, metrics_path, url, annotations)
        }
    }

    /// Starts the Crashpad handler process.
    ///
    /// # Arguments
    /// * `handler_path` - Path to the Crashpad handler executable
    /// * `database_path` - Path where crash dumps will be stored
    /// * `metrics_path` - Path for metrics data (can be empty)
    /// * `url` - URL to upload crash reports to (can be None for local-only)
    /// * `annotations` - Key-value pairs to include with crash reports
    pub fn start_handler(
        &self,
        handler_path: &Path,
        database_path: &Path,
        metrics_path: &Path,
        url: Option<&str>,
        annotations: &HashMap<String, String>,
    ) -> Result<()> {
        // Convert paths to C strings
        let handler_path_c = path_to_cstring(handler_path)?;
        let database_path_c = path_to_cstring(database_path)?;
        let metrics_path_c = path_to_cstring(metrics_path)?;

        let url_c = match url {
            Some(u) => Some(
                CString::new(u)
                    .map_err(|_| CrashpadError::InvalidConfiguration("Invalid URL".to_string()))?,
            ),
            None => None,
        };

        // Convert annotations to C-compatible arrays
        let mut keys: Vec<CString> = Vec::new();
        let mut values: Vec<CString> = Vec::new();

        for (k, v) in annotations {
            keys.push(CString::new(k.as_str()).map_err(|_| {
                CrashpadError::InvalidConfiguration("Invalid annotation key".to_string())
            })?);
            values.push(CString::new(v.as_str()).map_err(|_| {
                CrashpadError::InvalidConfiguration("Invalid annotation value".to_string())
            })?);
        }

        // Convert to raw pointers
        let keys_ptrs: Vec<*const std::os::raw::c_char> = keys.iter().map(|k| k.as_ptr()).collect();
        let values_ptrs: Vec<*const std::os::raw::c_char> =
            values.iter().map(|v| v.as_ptr()).collect();

        let success = unsafe {
            crashpad_client_start_handler(
                self.handle,
                handler_path_c.as_ptr(),
                database_path_c.as_ptr(),
                metrics_path_c.as_ptr(),
                url_c.as_ref().map_or(ptr::null(), |u| u.as_ptr()),
                keys_ptrs.as_ptr() as *mut *const std::os::raw::c_char,
                values_ptrs.as_ptr() as *mut *const std::os::raw::c_char,
                annotations.len(),
            )
        };

        if success {
            Ok(())
        } else {
            Err(CrashpadError::HandlerStartFailed)
        }
    }

    /// Starts the in-process handler (iOS/tvOS/watchOS only).
    #[cfg(any(target_os = "ios", target_os = "tvos", target_os = "watchos"))]
    fn start_in_process_handler(
        &self,
        database_path: &Path,
        metrics_path: &Path,
        url: Option<&str>,
        annotations: &HashMap<String, String>,
    ) -> Result<()> {
        // Convert paths to C strings
        let database_path_c = path_to_cstring(database_path)?;
        let _metrics_path_c = path_to_cstring(metrics_path)?;

        let url_c = match url {
            Some(u) => Some(
                CString::new(u)
                    .map_err(|_| CrashpadError::InvalidConfiguration("Invalid URL".to_string()))?,
            ),
            None => None,
        };

        // Convert annotations to C-compatible arrays
        let mut keys: Vec<CString> = Vec::new();
        let mut values: Vec<CString> = Vec::new();

        for (k, v) in annotations {
            keys.push(CString::new(k.as_str()).map_err(|_| {
                CrashpadError::InvalidConfiguration("Invalid annotation key".to_string())
            })?);
            values.push(CString::new(v.as_str()).map_err(|_| {
                CrashpadError::InvalidConfiguration("Invalid annotation value".to_string())
            })?);
        }

        // Convert to raw pointers
        let keys_ptrs: Vec<*const std::os::raw::c_char> = keys.iter().map(|k| k.as_ptr()).collect();
        let values_ptrs: Vec<*const std::os::raw::c_char> =
            values.iter().map(|v| v.as_ptr()).collect();

        // For iOS, we start the in-process handler
        let success = unsafe {
            crashpad_rs_sys::crashpad_client_start_in_process_handler(
                self.handle,
                database_path_c.as_ptr(),
                url_c.as_ref().map_or(ptr::null(), |u| u.as_ptr()),
                keys_ptrs.as_ptr() as *mut *const std::os::raw::c_char,
                values_ptrs.as_ptr() as *mut *const std::os::raw::c_char,
                annotations.len(),
            )
        };

        if success {
            // Start processing pending reports first
            unsafe {
                crashpad_rs_sys::crashpad_client_start_processing_pending_reports();
            }

            // Then process any intermediate dumps from previous sessions
            // This needs to be called after StartProcessingPendingReports
            unsafe {
                crashpad_rs_sys::crashpad_client_process_intermediate_dumps();
            }
            Ok(())
        } else {
            Err(CrashpadError::HandlerStartFailed)
        }
    }

    /// Sets the handler IPC pipe (Windows only).
    #[cfg(target_os = "windows")]
    pub fn set_handler_ipc_pipe(&self, ipc_pipe: &str) -> Result<()> {
        use std::ffi::OsStr;
        use std::os::windows::ffi::OsStrExt;

        let wide: Vec<u16> = OsStr::new(ipc_pipe).encode_wide().chain(Some(0)).collect();

        let success = unsafe { crashpad_client_set_handler_ipc_pipe(self.handle, wide.as_ptr()) };

        if success {
            Ok(())
        } else {
            Err(CrashpadError::HandlerStartFailed)
        }
    }

    /// Sets the handler Mach service (macOS/iOS only).
    #[cfg(any(target_os = "macos", target_os = "ios"))]
    pub fn set_handler_mach_service(&self, service_name: &str) -> Result<()> {
        let service_name_c = CString::new(service_name)
            .map_err(|_| CrashpadError::InvalidConfiguration("Invalid service name".to_string()))?;

        let success = unsafe {
            crashpad_client_set_handler_mach_service(self.handle, service_name_c.as_ptr())
        };

        if success {
            Ok(())
        } else {
            Err(CrashpadError::HandlerStartFailed)
        }
    }

    /// Use system default crash handler (macOS only).
    #[cfg(target_os = "macos")]
    pub fn use_system_default_handler(&self) -> Result<()> {
        let success = unsafe { crashpad_client_use_system_default_handler(self.handle) };

        if success {
            Ok(())
        } else {
            Err(CrashpadError::HandlerStartFailed)
        }
    }

    /// Process intermediate dumps (iOS only).
    ///
    /// Converts intermediate dumps to minidumps. This should be called:
    /// - On app startup to process crashes from previous sessions
    /// - After StartProcessingPendingReports has been called
    #[cfg(any(target_os = "ios", target_os = "tvos", target_os = "watchos"))]
    pub fn process_intermediate_dumps(&self) {
        unsafe {
            crashpad_sys::crashpad_client_process_intermediate_dumps();
        }
    }
}

impl Drop for CrashpadClient {
    fn drop(&mut self) {
        unsafe {
            crashpad_client_delete(self.handle);
        }
    }
}

// Send and Sync are safe because the C++ CrashpadClient is thread-safe
unsafe impl Send for CrashpadClient {}
unsafe impl Sync for CrashpadClient {}

fn path_to_cstring(path: &Path) -> Result<CString> {
    let path_str = path
        .to_str()
        .ok_or_else(|| CrashpadError::InvalidConfiguration("Invalid path".to_string()))?;
    CString::new(path_str)
        .map_err(|_| CrashpadError::InvalidConfiguration("Path contains null byte".to_string()))
}

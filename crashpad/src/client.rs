use std::ffi::CString;
use std::path::Path;
use std::ptr;
use std::collections::HashMap;

use crate::{Result, CrashpadError};

// Import FFI bindings
use crashpad_sys::*;

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
            Some(u) => Some(CString::new(u).map_err(|_| {
                CrashpadError::InvalidConfiguration("Invalid URL".to_string())
            })?),
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
        let keys_ptrs: Vec<*const std::os::raw::c_char> = keys.iter()
            .map(|k| k.as_ptr())
            .collect();
        let values_ptrs: Vec<*const std::os::raw::c_char> = values.iter()
            .map(|v| v.as_ptr())
            .collect();
        
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
    
    /// Sets the handler IPC pipe (Windows only).
    #[cfg(target_os = "windows")]
    pub fn set_handler_ipc_pipe(&self, ipc_pipe: &str) -> Result<()> {
        use std::os::windows::ffi::OsStrExt;
        use std::ffi::OsStr;
        
        let wide: Vec<u16> = OsStr::new(ipc_pipe)
            .encode_wide()
            .chain(Some(0))
            .collect();
        
        let success = unsafe {
            crashpad_client_set_handler_ipc_pipe(self.handle, wide.as_ptr())
        };
        
        if success {
            Ok(())
        } else {
            Err(CrashpadError::HandlerStartFailed)
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
    let path_str = path.to_str()
        .ok_or_else(|| CrashpadError::InvalidConfiguration("Invalid path".to_string()))?;
    CString::new(path_str)
        .map_err(|_| CrashpadError::InvalidConfiguration("Path contains null byte".to_string()))
}
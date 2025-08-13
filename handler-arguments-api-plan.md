# Handler Arguments API Implementation Plan

## Task ID: T-007
**Agent**: architect-loki  
**Date**: 2025-08-13  
**Status**: Planning

## 🎯 Goal
Enable Rust API to pass any command-line arguments to Crashpad handler process with both high-level and low-level interfaces.

## 📊 Problem Analysis
- [x] Current C++ wrapper passes empty arguments vector
- [x] FFI interface lacks arguments parameter
- [x] Many handler options not exposed
- [x] Need extensible interface design

## 🏗️ Implementation Phases

### Phase 1: FFI Layer Modification
- [ ] Add extra_arguments parameter to wrapper.h
- [ ] Modify crashpad_wrapper.cc to pass arguments
- [ ] Verify iOS in-process handler arguments support
- [ ] Handle NULL safety

### Phase 2: Rust Config Extension
- [ ] Add handler_arguments: Vec<String> field to CrashpadConfig
- [ ] Implement high-level API: rate_limit(), upload_gzip()
- [ ] Implement low-level API: handler_argument(), handler_arguments()
- [ ] Handle default values

### Phase 3: Client Implementation
- [ ] Convert Config handler_arguments to C string array
- [ ] Pass extra arguments through FFI
- [ ] Ensure memory safety
- [ ] Platform-specific handling (iOS may ignore some options)

### Phase 4: Testing
- [ ] Test high-level API (rate_limit, gzip)
- [ ] Test low-level API (arbitrary arguments)
- [ ] Test empty arguments
- [ ] Test arguments with special characters

### Phase 5: Documentation
- [ ] Update README.md Known Limitations section
- [ ] Add API documentation (list all possible options)
- [ ] Add example code (both high/low level)
- [ ] Create migration guide

## 📝 Implementation Details

### 1. FFI Interface (wrapper.h)
```c
bool crashpad_client_start_handler(
    crashpad_client_t client,
    const char* handler_path,
    const char* database_path,
    const char* metrics_path,
    const char* url,
    const char** annotations_keys,
    const char** annotations_values,
    size_t annotations_count,
    const char** extra_arguments,    // NEW
    size_t extra_arguments_count);   // NEW
```

### 2. C++ Implementation (crashpad_wrapper.cc)
```cpp
bool crashpad_client_start_handler(
    // ... existing parameters ...
    const char** extra_arguments,
    size_t extra_arguments_count) {
    
    // ... existing code ...
    
    std::vector<std::string> arguments;
    
    // Add extra arguments from caller
    for (size_t i = 0; i < extra_arguments_count; i++) {
        if (extra_arguments[i]) {
            arguments.push_back(extra_arguments[i]);
        }
    }
    
    return crashpad_client->StartHandler(
        // ... other params ...
        arguments,
        // ...
    );
}
```

### 3. Rust Config API (config.rs)
```rust
pub struct CrashpadConfig {
    // existing fields...
    handler_arguments: Vec<String>,
}

impl Default for CrashpadConfig {
    fn default() -> Self {
        Self {
            // ... existing defaults ...
            handler_arguments: Vec::new(),
        }
    }
}

impl CrashpadConfigBuilder {
    /// High-level API: Control rate limiting (default: true)
    pub fn rate_limit(mut self, enabled: bool) -> Self {
        if !enabled {
            self.config.handler_arguments.push("--no-rate-limit".to_string());
        }
        self
    }
    
    /// High-level API: Control gzip compression (default: true)
    pub fn upload_gzip(mut self, enabled: bool) -> Self {
        if !enabled {
            self.config.handler_arguments.push("--no-upload-gzip".to_string());
        }
        self
    }
    
    /// High-level API: Control periodic tasks (default: true)
    pub fn periodic_tasks(mut self, enabled: bool) -> Self {
        if !enabled {
            self.config.handler_arguments.push("--no-periodic-tasks".to_string());
        }
        self
    }
    
    /// High-level API: Control client identification in URL (default: true)
    pub fn identify_client_via_url(mut self, enabled: bool) -> Self {
        if !enabled {
            self.config.handler_arguments.push("--no-identify-client-via-url".to_string());
        }
        self
    }
    
    /// Low-level API: Add any handler argument
    pub fn handler_argument<S: Into<String>>(mut self, arg: S) -> Self {
        self.config.handler_arguments.push(arg.into());
        self
    }
    
    /// Low-level API: Add multiple handler arguments
    pub fn handler_arguments<I, S>(mut self, args: I) -> Self 
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.config.handler_arguments.extend(
            args.into_iter().map(Into::into)
        );
        self
    }
}
```

### 4. Client Implementation (client.rs)
```rust
pub fn start_handler(
    &self,
    handler_path: &Path,
    database_path: &Path,
    metrics_path: &Path,
    url: Option<&str>,
    annotations: &HashMap<String, String>,
) -> Result<()> {
    // ... existing code for paths and annotations ...
    
    // Convert handler arguments to C strings
    let handler_args: Vec<CString> = self.config.handler_arguments
        .iter()
        .map(|arg| CString::new(arg.as_str()))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| CrashpadError::InvalidConfiguration(
            "Handler argument contains null byte".to_string()
        ))?;
    
    let handler_args_ptrs: Vec<*const c_char> = handler_args
        .iter()
        .map(|arg| arg.as_ptr())
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
            handler_args_ptrs.as_ptr(),     // NEW
            handler_args_ptrs.len(),         // NEW
        )
    };
    
    // ... rest of code ...
}
```

## 📚 Supported Handler Options

### High-Level API (Common Options)
| Method | Handler Flag | Default | Description |
|--------|-------------|---------|-------------|
| `rate_limit(false)` | `--no-rate-limit` | true | Upload rate limiting (1/hour) |
| `upload_gzip(false)` | `--no-upload-gzip` | true | Gzip compression for uploads |
| `periodic_tasks(false)` | `--no-periodic-tasks` | true | Database scanning and pruning |
| `identify_client_via_url(false)` | `--no-identify-client-via-url` | true | Add client info to URL |

### Low-Level API (All Options)
```rust
// Example: Enable self-monitoring
let config = CrashpadConfig::builder()
    .handler_argument("--monitor-self")
    .handler_argument("--monitor-self-annotation=version=1.0")
    .build();

// Example: Custom combination
let config = CrashpadConfig::builder()
    .rate_limit(false)                                    // High-level
    .handler_argument("--no-write-minidump-to-database")  // Low-level
    .build();
```

### Complete List of Handler Options
- `--annotation=KEY=VALUE`: Process-level annotations
- `--monitor-self`: Run second handler to catch crashes in first
- `--monitor-self-annotation=KEY=VALUE`: Annotations for self-monitoring
- `--monitor-self-argument=ARGUMENT`: Arguments for monitor handler
- `--no-identify-client-via-url`: Don't add client fields to URL
- `--no-periodic-tasks`: Don't scan for new reports or prune database
- `--no-rate-limit`: Don't rate limit uploads
- `--no-upload-gzip`: Don't compress uploads
- `--no-write-minidump-to-database`: (Android only)
- Platform-specific options...

## ⚠️ Considerations
- [ ] Maintain complete backward compatibility
- [ ] iOS in-process handler may ignore some options
- [ ] Handler validates arguments (wrapper just passes through)
- [ ] Memory safety: CString lifetime must outlive FFI call
- [ ] Some options conflict with each other (handler will handle)

## ✅ Completion Criteria
- [ ] All current and future handler options can be passed
- [ ] High-level API covers 80% use cases
- [ ] Low-level API provides complete flexibility
- [ ] Existing code continues to work unchanged
- [ ] Test coverage > 90%
- [ ] Documentation includes all options
- [ ] README Known Limitations updated

## 🔄 Progress Tracking
- [ ] FFI interface design approved
- [ ] Implementation started
- [ ] Unit tests written
- [ ] Integration tests written
- [ ] Code review passed
- [ ] Documentation updated
- [ ] PR created and merged

## 📖 Usage Examples

### Basic Usage (High-Level API)
```rust
use crashpad_rs::{CrashpadClient, CrashpadConfig};

// Disable rate limiting for development
let config = CrashpadConfig::builder()
    .database_path("./crashes")
    .rate_limit(false)  // No upload throttling
    .build();

let client = CrashpadClient::new()?;
client.start_with_config(&config, &annotations)?;
```

### Advanced Usage (Mixed API)
```rust
// Production configuration with monitoring
let config = CrashpadConfig::builder()
    .database_path("/var/crash/myapp")
    .url("https://crashes.example.com")
    .rate_limit(true)           // Keep default rate limiting
    .upload_gzip(true)          // Keep compression
    .periodic_tasks(false)      // Disable periodic tasks (handled elsewhere)
    .handler_argument("--monitor-self")  // Add self-monitoring
    .build();
```

### Complete Control (Low-Level API)
```rust
// Full control over handler arguments
let config = CrashpadConfig::builder()
    .database_path("./crashes")
    .handler_arguments(vec![
        "--no-rate-limit",
        "--no-upload-gzip",
        "--monitor-self",
        "--monitor-self-annotation=env=prod",
        "--monitor-self-annotation=version=2.0",
    ])
    .build();
```

## 🔗 Related Files
- TASKS.md: T-007 task definition
- PRD.md: R-009 requirement
- wrapper.h: FFI interface
- crashpad_wrapper.cc: C++ implementation
- config.rs: Rust configuration API
- client.rs: Rust client implementation
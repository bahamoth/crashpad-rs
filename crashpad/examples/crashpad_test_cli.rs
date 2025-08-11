//! Crashpad Test CLI
//!
//! Interactive command-line tool for testing Crashpad crash reporting functionality.
//! Allows you to trigger crashes and verify that Crashpad captures them properly.

// Android standalone executables need special handling
// Due to -nodefaultlibs flag, we need to provide our own pthread_atfork
#[cfg(target_os = "android")]
#[no_mangle]
pub extern "C" fn pthread_atfork(
    _prepare: Option<extern "C" fn()>,
    _parent: Option<extern "C" fn()>,
    _child: Option<extern "C" fn()>,
) -> i32 {
    // Dummy implementation for testing
    // In a real app, this would be provided by the Android runtime
    0
}

use crashpad_rs::{CrashpadClient, CrashpadConfig};
use std::collections::HashMap;
use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Crashpad Test CLI");
    println!("=================");
    println!("Interactive tool for testing Crashpad crash reporting\n");

    // Create a new Crashpad client
    let client = CrashpadClient::new()?;
    println!("✓ Created Crashpad client");

    // Configure Crashpad with idiomatic builder pattern
    // Handler is now copied to target/{profile}/ or target/{target}/{profile}/
    let handler_path = if cfg!(target_os = "android") {
        // On Android, handler needs lib prefix and .so extension for APK
        let exe_path = std::env::current_exe()?;
        let exe_dir = exe_path.parent().unwrap();
        exe_dir.join("libcrashpad_handler.so")
    } else {
        // On desktop platforms, handler is in parent directory (target/debug/)
        // while examples are in target/debug/examples/
        let exe_path = std::env::current_exe()?;
        let exe_dir = exe_path.parent().unwrap();

        // Check if we're in examples directory
        if exe_dir.file_name() == Some(std::ffi::OsStr::new("examples")) {
            // Go up one level to find handler
            let handler_name = if cfg!(windows) {
                "crashpad_handler.exe"
            } else {
                "crashpad_handler"
            };
            exe_dir.parent().unwrap().join(handler_name)
        } else {
            // Same directory
            let handler_name = if cfg!(windows) {
                "crashpad_handler.exe"
            } else {
                "crashpad_handler"
            };
            exe_dir.join(handler_name)
        }
    };

    println!("Using handler at: {}", handler_path.display());

    let exe_path = std::env::current_exe()?;
    let exe_dir = exe_path.parent().unwrap();
    let config = CrashpadConfig::builder()
        .handler_path(handler_path)
        .database_path(exe_dir.join("crashpad_database"))
        .metrics_path(exe_dir.join("crashpad_metrics"))
        // .url("https://your-crash-server.com/submit")  // Optional
        .build();

    // Create annotations (metadata that will be included with crash reports)
    let mut annotations = HashMap::new();
    annotations.insert("version".to_string(), env!("CARGO_PKG_VERSION").to_string());
    annotations.insert("tool".to_string(), "crashpad_test_cli".to_string());
    annotations.insert(
        "platform".to_string(),
        format!("{}-{}", env::consts::OS, env::consts::ARCH),
    );
    annotations.insert("test_type".to_string(), "interactive".to_string());

    // Start the handler
    println!("Attempting to start handler...");
    match client.start_with_config(&config, &annotations) {
        Ok(_) => {
            println!("✓ Handler started successfully");
            println!("\nCrashpad is now monitoring this process for crashes.");
            println!("Any crashes will be saved to: ./crashpad_database");
        }
        Err(e) => {
            eprintln!("✗ Failed to start handler: {e}");
            eprintln!("\nTips:");
            eprintln!("- Set CRASHPAD_HANDLER environment variable to handler path");
            eprintln!("- Or ensure crashpad_handler is in the same directory as this executable");
            eprintln!("- Or install crashpad system-wide");
        }
    }

    // The client will remain active for the lifetime of this variable
    // When it's dropped, the handler connection will be closed

    // Check command line arguments or environment variables
    let args: Vec<String> = env::args().collect();
    let should_crash = args.len() > 1 && args[1] == "crash";
    let should_crash_env = env::var("CRASHPAD_TEST_CRASH").is_ok();

    if should_crash || should_crash_env {
        println!("\nTriggering crash now...");

        // Trigger an actual crash
        unsafe {
            // Null pointer dereference
            let null_ptr: *const i32 = std::ptr::null();
            println!("About to crash with value: {}", *null_ptr);
        }
    } else {
        println!("\nCrashpad initialized successfully!");
        println!("To trigger a crash, run with:");
        println!("  {} crash", args[0]);
        println!("Or set environment variable:");
        println!("  CRASHPAD_TEST_CRASH=1 {}", args[0]);
    }

    Ok(())
}

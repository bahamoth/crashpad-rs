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

use crashpad::{CrashpadClient, CrashpadConfig};
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
    let handler_path = if cfg!(debug_assertions) {
        // Development: explicitly specify the path
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        // CARGO_MANIFEST_DIR is the crashpad crate directory
        let workspace_root = std::path::Path::new(manifest_dir)
            .parent() // crashpad -> workspace root
            .unwrap();

        let platform = format!("{}-{}", env::consts::OS, env::consts::ARCH);

        let handler_name = if cfg!(target_os = "windows") {
            "crashpad_handler.exe"
        } else {
            "crashpad_handler"
        };

        let handler_path = workspace_root
            .join("third_party/crashpad_checkout/crashpad/out")
            .join(&platform)
            .join(handler_name);

        println!(
            "Development mode: using handler at {}",
            handler_path.display()
        );
        handler_path
    } else {
        // Production: expect handler in same directory as executable
        let exe_path = std::env::current_exe()?;
        let exe_dir = exe_path.parent().unwrap();
        let handler_name = if cfg!(target_os = "windows") {
            "crashpad_handler.exe"
        } else {
            "crashpad_handler"
        };
        exe_dir.join(handler_name)
    };

    let config = CrashpadConfig::builder()
        .handler_path(handler_path)
        .database_path("./crashpad_database")
        .metrics_path("./crashpad_metrics")
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

    // Keep the process running for a bit to demonstrate it's working
    println!("\nPress Enter to trigger a crash, or Ctrl+C to exit safely...");

    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;

    println!("Triggering crash in 1 second...");
    std::thread::sleep(std::time::Duration::from_secs(1));

    // Trigger an actual crash
    unsafe {
        // Null pointer dereference
        let null_ptr: *const i32 = std::ptr::null();
        println!("About to crash with value: {}", *null_ptr);
    }

    Ok(())
}

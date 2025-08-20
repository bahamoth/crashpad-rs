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
use std::process;

// Exit codes for different scenarios
const EXIT_SUCCESS: i32 = 0;
const EXIT_INIT_FAILED: i32 = 1;
const EXIT_HANDLER_FAILED: i32 = 2;
const EXIT_TEST_FAILED: i32 = 3;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Crashpad Test CLI");
    println!("=================");
    println!("Interactive tool for testing Crashpad crash reporting\n");

    // Create a new Crashpad client
    let client = match CrashpadClient::new() {
        Ok(c) => {
            println!("✓ Created Crashpad client");
            c
        }
        Err(e) => {
            eprintln!("✗ Failed to create Crashpad client: {e}");
            process::exit(EXIT_INIT_FAILED);
        }
    };

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
            process::exit(EXIT_HANDLER_FAILED);
        }
    }

    // The client will remain active for the lifetime of this variable
    // When it's dropped, the handler connection will be closed

    // Check command line arguments
    let args: Vec<String> = env::args().collect();
    let command = args.get(1).map(|s| s.as_str());

    match command {
        Some("dump") => {
            println!("\nCapturing dump without crash...");
            client.dump_without_crash();
            println!("✓ Dump captured successfully without crashing the process");
            println!("Check the database directory for the dump file:");
            println!("  {}", exe_dir.join("crashpad_database").display());
            if cfg!(windows) {
                println!("  On Windows, check the reports/ subdirectory");
            }
            process::exit(EXIT_SUCCESS);
        }
        Some("crash") => {
            println!("\nTriggering crash now...");

            // Windows needs a different approach to trigger SEH exceptions
            // that Crashpad can capture
            #[cfg(windows)]
            // SAFETY: This intentionally triggers a crash for testing purposes.
            // The inline assembly causes an access violation that will be caught
            // by Windows SEH and handled by Crashpad.
            unsafe {
                // Use inline assembly to bypass Rust's UB checks
                // This directly dereferences an invalid address, causing an access violation
                #[cfg(target_arch = "x86_64")]
                {
                    std::arch::asm!("mov dword ptr [0x7], 42", options(nostack));
                }
                #[cfg(target_arch = "x86")]
                {
                    std::arch::asm!("mov dword ptr [0x7], 42", options(nostack));
                }
                #[cfg(target_arch = "aarch64")]
                {
                    let invalid_ptr = 7 as *mut i32;
                    core::ptr::write_volatile(invalid_ptr, 42);
                }
            }

            #[cfg(not(windows))]
            // SAFETY: This intentionally triggers a crash for testing purposes.
            // The null pointer dereference will cause a segmentation fault
            // that will be caught by the signal handler and processed by Crashpad.
            unsafe {
                // On Unix-like systems, null pointer dereference works fine
                let null_ptr: *const i32 = std::ptr::null();
                println!("About to crash with value: {}", *null_ptr);
            }
        }
        Some("test") => {
            // Automated test mode with TAP output
            println!("\n# TAP version 13");
            println!("1..3");

            // Test 1: Initialization
            println!("ok 1 - Crashpad client created");

            // Test 2: Handler started
            println!("ok 2 - Handler started successfully");

            // Test 3: Configuration validated
            let db_path = exe_dir.join("crashpad_database");
            if db_path.exists() || std::fs::create_dir_all(&db_path).is_ok() {
                println!("ok 3 - Database directory accessible");
            } else {
                println!("not ok 3 - Database directory not accessible");
                process::exit(EXIT_TEST_FAILED);
            }

            println!("\n# All tests passed");
            process::exit(EXIT_SUCCESS);
        }
        Some("--help") | Some("-h") => {
            println!("\nUsage: {} [COMMAND]", args[0]);
            println!("\nCommands:");
            println!("  dump     Capture a dump without crashing");
            println!("  crash    Trigger a crash to test handler");
            println!("  test     Run automated tests with TAP output");
            println!("  --help   Show this help message");
            println!("\nEnvironment variables:");
            println!("  CRASHPAD_HANDLER   Path to crashpad_handler executable");
            println!(
                "  CRASHPAD_TEST_CRASH   Set to trigger crash (deprecated, use 'crash' command)"
            );
        }
        _ => {
            // Interactive mode (default)
            let should_crash_env = env::var("CRASHPAD_TEST_CRASH").is_ok();

            if should_crash_env {
                println!("\nTriggering crash due to CRASHPAD_TEST_CRASH environment variable...");
                // SAFETY: This intentionally triggers a crash for testing purposes.
                // The null pointer dereference will cause a segmentation fault.
                unsafe {
                    let null_ptr: *const i32 = std::ptr::null();
                    println!("About to crash with value: {}", *null_ptr);
                }
            } else {
                println!("\nCrashpad initialized successfully!");
                println!("Available commands:");
                println!("  {} dump     - Capture a dump without crashing", args[0]);
                println!("  {} crash    - Trigger a crash to test handler", args[0]);
                println!("  {} test     - Run automated tests", args[0]);
                println!("\nOr set environment variable:");
                println!("  CRASHPAD_TEST_CRASH=1 {} (deprecated)", args[0]);
            }
        }
    }

    Ok(())
}

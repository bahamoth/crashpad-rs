//! iOS Simulator test example for Crashpad
//!
//! iOS uses an in-process handler, so there's no separate handler executable.
//! The Crashpad client operates within the same process.

#[cfg(any(target_os = "ios", target_os = "tvos", target_os = "watchos"))]
use crashpad_rs::{CrashpadClient, CrashpadConfig};
#[cfg(any(target_os = "ios", target_os = "tvos", target_os = "watchos"))]
use std::collections::HashMap;

#[cfg(not(any(target_os = "ios", target_os = "tvos", target_os = "watchos")))]
fn main() {
    eprintln!("This example is only for iOS/tvOS/watchOS platforms");
    std::process::exit(1);
}

#[cfg(any(target_os = "ios", target_os = "tvos", target_os = "watchos"))]
fn main() {
    println!("iOS Crashpad Test (In-Process Handler)");
    println!("======================================");

    // Create Crashpad client
    let client = match CrashpadClient::new() {
        Ok(c) => {
            println!("✓ Created Crashpad client");
            c
        }
        Err(e) => {
            eprintln!("✗ Failed to create client: {}", e);
            return;
        }
    };

    // For iOS, we don't need a handler path since it's in-process
    // Just configure database and metrics paths
    let config = CrashpadConfig::builder()
        .database_path("./crashpad_database")
        .metrics_path("./crashpad_metrics")
        .build();

    // Prepare annotations
    let mut annotations = HashMap::new();
    annotations.insert("platform".to_string(), "ios-simulator".to_string());
    annotations.insert("version".to_string(), env!("CARGO_PKG_VERSION").to_string());
    annotations.insert("test_type".to_string(), "in_process".to_string());

    // Start the in-process handler
    match client.start_with_config(&config, &annotations) {
        Ok(_) => println!("✓ Started in-process handler"),
        Err(e) => {
            eprintln!("✗ Failed to start in-process handler: {}", e);
            return;
        }
    }

    // Explicitly process intermediate dumps again
    println!("Processing intermediate dumps...");
    client.process_intermediate_dumps();
    println!("✓ Processed intermediate dumps");

    // Check command line arguments or environment variables for CI mode
    let args: Vec<String> = std::env::args().collect();
    let should_crash = args.len() > 1 && args[1] == "crash";
    let should_crash_env = std::env::var("CRASHPAD_TEST_CRASH").is_ok();

    if should_crash || should_crash_env {
        println!("\nTriggering crash now...");

        // Trigger an actual crash
        unsafe {
            // Null pointer dereference
            let null_ptr: *const i32 = std::ptr::null();
            println!("About to crash with value: {}", *null_ptr);
        }
    } else {
        println!("\niOS Crashpad initialized successfully!");
        println!("In-process handler is active and monitoring for crashes.");
        println!("To trigger a crash, run with:");
        println!("  {} crash", args[0]);
        println!("Or set environment variable:");
        println!("  CRASHPAD_TEST_CRASH=1 {}", args[0]);
    }

    println!("Test completed (you shouldn't see this after a crash)");
}

#[cfg(any(target_os = "ios", target_os = "tvos", target_os = "watchos"))]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ios_client_creation() {
        // Just verify we can create a client on iOS
        let client = CrashpadClient::new();
        assert!(client.is_ok(), "Should be able to create client on iOS");
    }

    #[test]
    fn test_ios_in_process_config() {
        let client = CrashpadClient::new().unwrap();

        // iOS doesn't need handler path
        let config = CrashpadConfig::builder()
            .database_path("/tmp/crashpad_database")
            .metrics_path("/tmp/crashpad_metrics")
            .build();

        let annotations = HashMap::new();

        // This might fail in test environment but should not panic
        let _ = client.start_with_config(&config, &annotations);
    }
}

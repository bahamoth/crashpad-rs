//! iOS Simulator test example for Crashpad
//!
//! iOS uses an in-process handler, so there's no separate handler executable.
//! The Crashpad client operates within the same process.

#[cfg(any(target_os = "ios", target_os = "tvos", target_os = "watchos"))]
use crashpad::{CrashpadClient, CrashpadConfig};
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

    println!("\nTest scenarios:");
    println!("1. Normal operation test");
    println!("2. Null pointer dereference");
    println!("3. Abort signal");
    println!("4. Exit normally");
    println!("5. Check crash dump status");

    // Actually trigger a crash for testing
    println!("\nTriggering crash for testing...");
    let choice = "2"; // Null pointer dereference

    match choice {
        "1" => {
            println!("Running normal operation test...");
            // Simulate some work
            for i in 0..5 {
                println!("Working... {}/5", i + 1);
                std::thread::sleep(std::time::Duration::from_secs(1));
            }
            println!("Normal operation completed successfully!");
        }
        "2" => {
            println!("Triggering null pointer dereference...");
            std::thread::sleep(std::time::Duration::from_secs(1));
            unsafe {
                let null_ptr: *const i32 = std::ptr::null();
                println!("Value at null: {}", *null_ptr); // This will crash
            }
        }
        "3" => {
            println!("Triggering abort...");
            std::thread::sleep(std::time::Duration::from_secs(1));
            std::process::abort();
        }
        "4" => {
            println!("Exiting normally...");
        }
        "5" => {
            println!("Checking crash dump directories...");
            println!("(ProcessIntermediateDumps was called during initialization)");
            println!("Any intermediate dumps should now be converted to minidumps.");
        }
        _ => {
            println!("Invalid choice, exiting...");
        }
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

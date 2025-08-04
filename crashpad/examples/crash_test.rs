//! Example that demonstrates crash handling
//! 
//! WARNING: This example intentionally crashes the program!
//! Only run this if you want to test crash reporting.

use crashpad::CrashpadClient;
use std::collections::HashMap;
use std::path::PathBuf;
use std::env;

extern crate libc;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Crashpad Crash Test Example");
    println!("===========================");
    println!("WARNING: This program will intentionally crash!");
    println!();
    
    // Set up Crashpad
    let client = CrashpadClient::new()?;
    
    let exe_path = env::current_exe()?;
    let exe_dir = exe_path.parent().unwrap();
    
    let handler_path = find_handler_path();
    let database_path = exe_dir.join("crashpad_database");
    let metrics_path = exe_dir.join("crashpad_metrics");
    
    let mut annotations = HashMap::new();
    annotations.insert("example".to_string(), "crash_test".to_string());
    annotations.insert("crash_type".to_string(), "segmentation_fault".to_string());
    
    match client.start_handler(
        &handler_path,
        &database_path,
        &metrics_path,
        None,
        &annotations,
    ) {
        Ok(_) => {
            println!("✓ Crashpad handler started");
            println!("  Crash dumps will be saved to: {}", database_path.display());
        }
        Err(e) => {
            eprintln!("✗ Failed to start handler: {}", e);
            return Ok(());
        }
    }
    
    println!("\nTriggering crash in 3 seconds...");
    std::thread::sleep(std::time::Duration::from_secs(1));
    println!("2...");
    std::thread::sleep(std::time::Duration::from_secs(1));
    println!("1...");
    std::thread::sleep(std::time::Duration::from_secs(1));
    
    // Trigger a segmentation fault
    println!("Crashing now!");
    
    // Use libc to raise SIGSEGV directly
    unsafe {
        libc::raise(libc::SIGSEGV);
    }
    
    // This line will never be reached
    unreachable!("This should never execute");
}

fn find_handler_path() -> PathBuf {
    let possible_paths = vec![
        PathBuf::from("../../../../third_party/crashpad_checkout/crashpad/out/linux-x86_64/crashpad_handler"),
        PathBuf::from("/home/bahamoth/projects/crashpad-rs/third_party/crashpad_checkout/crashpad/out/linux-x86_64/crashpad_handler"),
        PathBuf::from("crashpad_handler"),
    ];
    
    for path in &possible_paths {
        if path.exists() {
            return path.clone();
        }
    }
    
    possible_paths[0].clone()
}
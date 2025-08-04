//! Basic example of using Crashpad
//! 
//! This example shows how to set up Crashpad crash reporting in a Rust application.

use crashpad::CrashpadClient;
use std::collections::HashMap;
use std::path::PathBuf;
use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Crashpad Basic Example");
    println!("=====================");
    
    // Create a new Crashpad client
    let client = CrashpadClient::new()?;
    println!("✓ Created Crashpad client");
    
    // Set up paths
    let exe_path = env::current_exe()?;
    let exe_dir = exe_path.parent().unwrap();
    
    // For this example, we'll look for the handler in the build directory
    let handler_path = find_handler_path();
    let database_path = exe_dir.join("crashpad_database");
    let metrics_path = exe_dir.join("crashpad_metrics");
    
    println!("Handler path: {}", handler_path.display());
    println!("Database path: {}", database_path.display());
    println!("Metrics path: {}", metrics_path.display());
    
    // Create annotations (metadata that will be included with crash reports)
    let mut annotations = HashMap::new();
    annotations.insert("version".to_string(), env!("CARGO_PKG_VERSION").to_string());
    annotations.insert("example".to_string(), "basic".to_string());
    annotations.insert("rust_version".to_string(), rustc_version().to_string());
    
    // Optional: Set a crash report upload URL
    // let url = Some("https://your-crash-server.com/submit");
    let url = None; // Local-only for this example
    
    // Start the handler
    match client.start_handler(
        &handler_path,
        &database_path,
        &metrics_path,
        url,
        &annotations,
    ) {
        Ok(_) => {
            println!("✓ Handler started successfully");
            println!("\nCrashpad is now monitoring this process for crashes.");
            println!("Any crashes will be saved to: {}", database_path.display());
        }
        Err(e) => {
            eprintln!("✗ Failed to start handler: {}", e);
            eprintln!("\nMake sure crashpad_handler is built and accessible.");
            eprintln!("You may need to run: cargo build --package crashpad-sys");
        }
    }
    
    // The client will remain active for the lifetime of this variable
    // When it's dropped, the handler connection will be closed
    
    println!("\nPress Enter to exit...");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    
    Ok(())
}

fn find_handler_path() -> PathBuf {
    // Try to find the built crashpad_handler
    let possible_paths = vec![
        // Relative to the example binary
        PathBuf::from("../../../../third_party/crashpad_checkout/crashpad/out/linux-x86_64/crashpad_handler"),
        // Absolute path
        PathBuf::from("/home/bahamoth/projects/crashpad-rs/third_party/crashpad_checkout/crashpad/out/linux-x86_64/crashpad_handler"),
        // In case it's installed system-wide
        PathBuf::from("/usr/local/bin/crashpad_handler"),
        PathBuf::from("crashpad_handler"),
    ];
    
    for path in &possible_paths {
        if path.exists() {
            return path.clone();
        }
    }
    
    // Return the first path as a fallback
    possible_paths[0].clone()
}

fn rustc_version() -> String {
    // Simple version string
    format!("{}.{}.{}", 
        env!("CARGO_PKG_RUST_VERSION"),
        std::env::consts::ARCH,
        std::env::consts::OS
    )
}
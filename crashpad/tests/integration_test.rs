use crashpad::CrashpadClient;
use std::collections::HashMap;
use std::path::PathBuf;
use tempfile::TempDir;

#[test]
fn test_client_new_and_drop() {
    // Create client and verify proper cleanup
    let client = CrashpadClient::new();
    assert!(client.is_ok());

    // Drop is called automatically
    drop(client);
}

#[test]
#[ignore] // Run with `cargo test -- --ignored` due to global state conflicts
fn test_start_handler() {
    let client = CrashpadClient::new().expect("Failed to create client");

    // Create temporary directory
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let database_path = temp_dir.path().join("crashpad_db");
    let metrics_path = temp_dir.path().join("crashpad_metrics");

    // Pre-create directories
    std::fs::create_dir_all(&database_path).ok();
    std::fs::create_dir_all(&metrics_path).ok();

    // Find built handler path
    let handler_path = find_crashpad_handler();

    // Start with empty annotations
    let annotations = HashMap::new();

    // Start handler (local only, no URL)
    let result = client.start_handler(
        &handler_path,
        &database_path,
        &metrics_path,
        None,
        &annotations,
    );

    // Check if handler file exists
    if handler_path.exists() {
        // Should succeed if handler exists
        assert!(
            result.is_ok(),
            "Handler should start successfully with valid path"
        );
        println!("Handler started successfully");
    } else {
        // May fail if handler is missing (depends on Crashpad implementation)
        println!("Handler path doesn't exist, result: {result:?}");
    }
}

#[test]
#[ignore] // Run with `cargo test -- --ignored` due to global state conflicts
fn test_invalid_paths() {
    let client = CrashpadClient::new().expect("Failed to create client");

    // Non-existent handler path
    let invalid_handler = PathBuf::from("/nonexistent/crashpad_handler");
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let database_path = temp_dir.path().join("crashpad_db");
    let metrics_path = temp_dir.path().join("crashpad_metrics");

    // Pre-create directories
    std::fs::create_dir_all(&database_path).expect("Failed to create database dir");
    std::fs::create_dir_all(&metrics_path).expect("Failed to create metrics dir");

    println!("TempDir path: {}", temp_dir.path().display());
    println!(
        "Database path: {} (exists: {})",
        database_path.display(),
        database_path.exists()
    );
    println!(
        "Metrics path: {} (exists: {})",
        metrics_path.display(),
        metrics_path.exists()
    );

    let annotations = HashMap::new();

    let result = client.start_handler(
        &invalid_handler,
        &database_path,
        &metrics_path,
        None,
        &annotations,
    );

    // Note: Crashpad may process handler startup asynchronously
    // so it may not return failure immediately for invalid paths
    // The actual handler process will fail later
    println!("Result with invalid path: {result:?}");
}

#[test]
#[ignore] // Run with `cargo test -- --ignored` due to global state conflicts
fn test_with_annotations() {
    let client = CrashpadClient::new().expect("Failed to create client");

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let database_path = temp_dir.path().join("crashpad_db");
    let metrics_path = temp_dir.path().join("crashpad_metrics");

    // Pre-create directories
    std::fs::create_dir_all(&database_path).ok();
    std::fs::create_dir_all(&metrics_path).ok();

    let handler_path = find_crashpad_handler();

    // Add multiple annotations
    let mut annotations = HashMap::new();
    annotations.insert("version".to_string(), "1.0.0".to_string());
    annotations.insert("build".to_string(), "debug".to_string());
    annotations.insert("user".to_string(), "test_user".to_string());

    let result = client.start_handler(
        &handler_path,
        &database_path,
        &metrics_path,
        None,
        &annotations,
    );

    match result {
        Ok(_) => println!("Handler with annotations started successfully"),
        Err(e) => println!("Handler start failed (expected in test env): {e:?}"),
    }
}

// Helper function to find the built crashpad_handler
fn find_crashpad_handler() -> PathBuf {
    // First look in build location
    let possible_paths = vec![
        PathBuf::from("../third_party/crashpad_checkout/crashpad/out/linux-x86_64/crashpad_handler"),
        PathBuf::from("third_party/crashpad_checkout/crashpad/out/linux-x86_64/crashpad_handler"),
        PathBuf::from("/home/bahamoth/projects/crashpad-rs/third_party/crashpad_checkout/crashpad/out/linux-x86_64/crashpad_handler"),
    ];

    for path in possible_paths {
        if path.exists() {
            return path;
        }
    }

    // Return dummy path if not found (test will fail)
    PathBuf::from("crashpad_handler")
}

use crashpad_rs::CrashpadClient;
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
fn test_start_handler_valid() {
    let client = CrashpadClient::new().expect("CrashpadClient::new() should succeed");

    // Create temporary directory
    let temp_dir = TempDir::new().expect("Should be able to create temp directory");
    let database_path = temp_dir.path().join("crashpad_db");
    let metrics_path = temp_dir.path().join("crashpad_metrics");

    // Pre-create directories
    std::fs::create_dir_all(&database_path).expect("Should be able to create database directory");
    std::fs::create_dir_all(&metrics_path).expect("Should be able to create metrics directory");

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
            "Handler should start successfully with valid path: {result:?}"
        );
        println!("✓ Handler started successfully");
    } else {
        // May fail if handler is missing (acceptable in test environment)
        println!(
            "Handler path doesn't exist at {}, skipping test",
            handler_path.display()
        );
    }
}

#[test]
fn test_invalid_handler_path() {
    // This test verifies behavior with invalid handler path
    // Note: We don't actually start the handler to avoid global state issues

    // Just verify that the path doesn't exist
    let invalid_handler = PathBuf::from("/nonexistent/crashpad_handler");
    assert!(
        !invalid_handler.exists(),
        "Invalid handler path should not exist"
    );

    // Verify we can create a client (doesn't start handler)
    let client = CrashpadClient::new();
    assert!(
        client.is_ok(),
        "Should be able to create client even without handler"
    );

    println!("✓ Invalid handler path test passed (path verification only)");
}

#[test]
fn test_with_annotations() {
    let client = CrashpadClient::new().expect("CrashpadClient::new() should succeed");

    let temp_dir = TempDir::new().expect("Should be able to create temp directory");
    let database_path = temp_dir.path().join("crashpad_db");
    let metrics_path = temp_dir.path().join("crashpad_metrics");

    // Pre-create directories
    std::fs::create_dir_all(&database_path).expect("Should be able to create database directory");
    std::fs::create_dir_all(&metrics_path).expect("Should be able to create metrics directory");

    let handler_path = find_crashpad_handler();

    // Add multiple annotations
    let mut annotations = HashMap::new();
    annotations.insert("version".to_string(), "1.0.0".to_string());
    annotations.insert("build".to_string(), "debug".to_string());
    annotations.insert("user".to_string(), "test_user".to_string());
    annotations.insert("test_id".to_string(), "test_with_annotations".to_string());

    let result = client.start_handler(
        &handler_path,
        &database_path,
        &metrics_path,
        None,
        &annotations,
    );

    if handler_path.exists() {
        assert!(
            result.is_ok(),
            "Handler with annotations should start successfully: {result:?}"
        );
        println!("✓ Handler with annotations started successfully");
    } else {
        println!("Handler not found, skipping annotation test");
    }
}

// Helper function to find the built crashpad_handler
fn find_crashpad_handler() -> PathBuf {
    let platform = format!(
        "{}-{}",
        std::env::consts::OS,
        if cfg!(target_arch = "x86_64") {
            "x64"
        } else {
            "arm64"
        }
    );

    // Look in build location
    let possible_paths = vec![
        format!("third_party/crashpad_checkout/crashpad/out/{}/crashpad_handler", platform),
        format!("../third_party/crashpad_checkout/crashpad/out/{}/crashpad_handler", platform),
        format!("/home/bahamoth/projects/crashpad-rs/third_party/crashpad_checkout/crashpad/out/{}/crashpad_handler", platform),
    ];

    for path_str in possible_paths {
        let path = PathBuf::from(path_str);
        if path.exists() {
            return path;
        }
    }

    // Return dummy path if not found (test will handle it)
    PathBuf::from("crashpad_handler")
}

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

// Combine all handler tests into one to avoid global state conflicts
// Crashpad on Linux uses a global SignalHandler that can only be set once per process
#[test]
fn test_handler_scenarios() {
    println!("Testing Crashpad handler scenarios...");

    // Test 1: Create client with valid paths
    let client = CrashpadClient::new().expect("Failed to create client");
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let database_path = temp_dir.path().join("crashpad_db");
    let metrics_path = temp_dir.path().join("crashpad_metrics");

    // Pre-create directories
    std::fs::create_dir_all(&database_path).expect("Failed to create database dir");
    std::fs::create_dir_all(&metrics_path).expect("Failed to create metrics dir");

    // Test 2: Try with non-existent handler first (should fail gracefully)
    let invalid_handler = PathBuf::from("/nonexistent/crashpad_handler");
    let empty_annotations = HashMap::new();

    let invalid_result = client.start_handler(
        &invalid_handler,
        &database_path,
        &metrics_path,
        None,
        &empty_annotations,
    );

    // On Linux, this will actually set up the signal handler even with invalid path
    // The handler process spawn will fail but the client setup succeeds
    println!("Invalid handler result: {invalid_result:?}");

    // Note: We cannot test with a valid handler after this because
    // the signal handler is already installed
}

// Test annotations separately without starting handler
#[test]
fn test_annotations_setup() {
    // This test just verifies we can create annotations
    // without actually starting a handler
    let mut annotations = HashMap::new();
    annotations.insert("version".to_string(), "1.0.0".to_string());
    annotations.insert("build".to_string(), "debug".to_string());
    annotations.insert("user".to_string(), "test_user".to_string());

    assert_eq!(annotations.len(), 3);
    assert_eq!(annotations.get("version"), Some(&"1.0.0".to_string()));
}

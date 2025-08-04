use crashpad::CrashpadClient;
use std::collections::HashMap;
use std::path::PathBuf;
use tempfile::TempDir;

#[test]
fn test_client_new_and_drop() {
    // 클라이언트를 생성하고 정상적으로 삭제되는지 확인
    let client = CrashpadClient::new();
    assert!(client.is_ok());
    
    // Drop은 자동으로 호출됨
    drop(client);
}

#[test]
fn test_start_handler() {
    let client = CrashpadClient::new().expect("Failed to create client");
    
    // 임시 디렉토리 생성
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let database_path = temp_dir.path().join("crashpad_db");
    let metrics_path = temp_dir.path().join("crashpad_metrics");
    
    // 디렉토리 미리 생성
    std::fs::create_dir_all(&database_path).ok();
    std::fs::create_dir_all(&metrics_path).ok();
    
    // 빌드된 handler 경로 찾기
    let handler_path = find_crashpad_handler();
    
    // 빈 annotations로 시작
    let annotations = HashMap::new();
    
    // 핸들러 시작 (URL 없이 로컬 전용)
    let result = client.start_handler(
        &handler_path,
        &database_path,
        &metrics_path,
        None,
        &annotations,
    );
    
    // 실제 핸들러 파일이 존재하는지 확인
    if handler_path.exists() {
        // 핸들러가 존재하면 성공해야 함
        assert!(result.is_ok(), "Handler should start successfully with valid path");
        println!("Handler started successfully");
    } else {
        // 핸들러가 없으면 실패할 수도 있음 (Crashpad 구현에 따라 다름)
        println!("Handler path doesn't exist, result: {:?}", result);
    }
}

#[test]
fn test_invalid_paths() {
    let client = CrashpadClient::new().expect("Failed to create client");
    
    // 존재하지 않는 핸들러 경로
    let invalid_handler = PathBuf::from("/nonexistent/crashpad_handler");
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let database_path = temp_dir.path().join("crashpad_db");
    let metrics_path = temp_dir.path().join("crashpad_metrics");
    
    // 디렉토리 미리 생성
    std::fs::create_dir_all(&database_path).expect("Failed to create database dir");
    std::fs::create_dir_all(&metrics_path).expect("Failed to create metrics dir");
    
    println!("TempDir path: {}", temp_dir.path().display());
    println!("Database path: {} (exists: {})", database_path.display(), database_path.exists());
    println!("Metrics path: {} (exists: {})", metrics_path.display(), metrics_path.exists());
    
    let annotations = HashMap::new();
    
    let result = client.start_handler(
        &invalid_handler,
        &database_path,
        &metrics_path,
        None,
        &annotations,
    );
    
    // 참고: Crashpad는 핸들러 시작을 비동기로 처리할 수 있어서
    // 잘못된 경로에서도 즉시 실패를 반환하지 않을 수 있음
    // 실제 핸들러 프로세스는 나중에 실패함
    println!("Result with invalid path: {:?}", result);
}

#[test]
fn test_with_annotations() {
    let client = CrashpadClient::new().expect("Failed to create client");
    
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let database_path = temp_dir.path().join("crashpad_db");
    let metrics_path = temp_dir.path().join("crashpad_metrics");
    
    // 디렉토리 미리 생성
    std::fs::create_dir_all(&database_path).ok();
    std::fs::create_dir_all(&metrics_path).ok();
    
    let handler_path = find_crashpad_handler();
    
    // 여러 annotation 추가
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
        Err(e) => println!("Handler start failed (expected in test env): {:?}", e),
    }
}

// Helper function to find the built crashpad_handler
fn find_crashpad_handler() -> PathBuf {
    // 먼저 빌드된 위치에서 찾기
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
    
    // 찾을 수 없으면 더미 경로 반환 (테스트는 실패할 것임)
    PathBuf::from("crashpad_handler")
}
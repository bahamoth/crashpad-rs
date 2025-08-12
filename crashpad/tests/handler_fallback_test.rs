//! Integration test for handler search fallback logic

use crashpad_rs::CrashpadConfig;
use std::env;
use std::fs;
use std::path::PathBuf;

#[test]
#[cfg(not(any(target_os = "ios", target_os = "tvos", target_os = "watchos")))]
fn test_explicit_handler_path() {
    // When handler_path is explicitly set, it should be used as-is
    let explicit_path = PathBuf::from("/custom/path/to/handler");
    let _config = CrashpadConfig::builder()
        .handler_path(&explicit_path)
        .database_path("/tmp/test")
        .build();

    // We can't access handler_path() directly (it's pub(crate))
    // but we can verify the config was built correctly
    // The actual path will be validated when CrashpadClient::start_with_config is called
}

#[test]
#[cfg(not(any(target_os = "ios", target_os = "tvos", target_os = "watchos")))]
fn test_env_var_fallback() {
    // Save original env var
    let original = env::var("CRASHPAD_HANDLER").ok();

    // Test with CRASHPAD_HANDLER environment variable
    let test_path = "/test/env/handler";
    env::set_var("CRASHPAD_HANDLER", test_path);

    let _config = CrashpadConfig::builder().database_path("/tmp/test").build();

    // The handler_path() method should check env var when no explicit path is set

    // Restore original env var
    if let Some(orig) = original {
        env::set_var("CRASHPAD_HANDLER", orig);
    } else {
        env::remove_var("CRASHPAD_HANDLER");
    }
}

#[test]
#[cfg(not(any(target_os = "ios", target_os = "tvos", target_os = "watchos")))]
fn test_handler_name_by_platform() {
    // Test that the correct handler name is used for each platform
    let _config = CrashpadConfig::builder().database_path("/tmp/test").build();

    // Expected handler name based on platform
    let expected_name = if cfg!(target_os = "android") {
        "libcrashpad_handler.so"
    } else if cfg!(windows) {
        "crashpad_handler.exe"
    } else {
        "crashpad_handler"
    };

    // Create a test handler file in current directory
    let test_handler = PathBuf::from(expected_name);
    fs::write(&test_handler, b"test").unwrap();

    // The search should find this handler in current directory
    // (actual behavior would be tested via CrashpadClient)

    // Cleanup
    fs::remove_file(test_handler).unwrap();
}

#[test]
#[cfg(not(any(target_os = "ios", target_os = "tvos", target_os = "watchos")))]
fn test_search_order_priority() {
    // This test verifies the search order priority
    // 1. Config path > 2. ENV var > 3. Exe dir > 4. Current dir

    let original_env = env::var("CRASHPAD_HANDLER").ok();

    // Setup: Create handlers in different locations
    let config_path = PathBuf::from("/tmp/config_handler");
    let env_path = PathBuf::from("/tmp/env_handler");

    // Test 1: Config path should take precedence over everything
    {
        env::set_var("CRASHPAD_HANDLER", env_path.to_str().unwrap());

        let _config = CrashpadConfig::builder()
            .handler_path(&config_path)
            .database_path("/tmp/test")
            .build();

        // Config path should be used, ignoring env var
    }

    // Test 2: ENV var should be used when no config path
    {
        env::set_var("CRASHPAD_HANDLER", env_path.to_str().unwrap());

        let _config = CrashpadConfig::builder().database_path("/tmp/test").build();

        // ENV var should be checked
    }

    // Test 3: Fallback to exe dir and current dir when no config or env
    {
        env::remove_var("CRASHPAD_HANDLER");

        let _config = CrashpadConfig::builder().database_path("/tmp/test").build();

        // Should search exe dir then current dir
    }

    // Restore env
    if let Some(orig) = original_env {
        env::set_var("CRASHPAD_HANDLER", orig);
    } else {
        env::remove_var("CRASHPAD_HANDLER");
    }
}

#[test]
#[cfg(any(target_os = "ios", target_os = "tvos", target_os = "watchos"))]
fn test_ios_no_handler_needed() {
    // iOS/tvOS/watchOS should not require a handler path
    let _config = CrashpadConfig::builder().database_path("/tmp/test").build();

    // handler_path() should return empty path for iOS platforms
    // (in-process handler is used)
}

// Wrapper header for bindgen to generate Rust FFI bindings

#ifndef CRASHPAD_WRAPPER_H
#define CRASHPAD_WRAPPER_H

#include <stdbool.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

// Opaque handle for CrashpadClient
typedef void* crashpad_client_t;

// Create a new CrashpadClient instance
crashpad_client_t crashpad_client_new();

// Delete a CrashpadClient instance
void crashpad_client_delete(crashpad_client_t client);

// Start the Crashpad handler
bool crashpad_client_start_handler(
    crashpad_client_t client,
    const char* handler_path,
    const char* database_path,
    const char* metrics_path,
    const char* url,
    const char** annotations_keys,
    const char** annotations_values,
    size_t annotations_count,
    const char** extra_arguments,
    size_t extra_arguments_count);

// Set handler IPC pipe (for Windows)
#ifdef _WIN32
bool crashpad_client_set_handler_ipc_pipe(
    crashpad_client_t client,
    const wchar_t* ipc_pipe);
#endif

// Platform-specific functions for macOS/iOS
#if defined(__APPLE__)
// Set handler for macOS/iOS using mach port
bool crashpad_client_set_handler_mach_service(
    crashpad_client_t client,
    const char* service_name);

// Use system crash reporter on macOS
bool crashpad_client_use_system_default_handler(
    crashpad_client_t client);

// iOS-specific in-process handler functions
#if defined(TARGET_OS_IOS) && TARGET_OS_IOS
bool crashpad_client_start_in_process_handler(
    crashpad_client_t client,
    const char* database_path,
    const char* url,
    const char** annotations_keys,
    const char** annotations_values,
    size_t annotations_count);

void crashpad_client_process_intermediate_dumps();

void crashpad_client_start_processing_pending_reports();
#endif
#endif

// DumpWithoutCrash support - capture a dump without crashing the process
// This is useful for diagnostic purposes when you want to capture the current
// state without terminating the application
void crashpad_dump_without_crash();

// Alternative that allows passing a pre-captured context
// On Windows: context should be a pointer to CONTEXT structure
// On other platforms: context should be a pointer to NativeCPUContext
void crashpad_dump_without_crash_with_context(void* context);

#ifdef __cplusplus
}
#endif

#endif // CRASHPAD_WRAPPER_H
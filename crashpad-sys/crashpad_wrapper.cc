#include "client/crashpad_client.h"
#include <memory>

#ifdef _WIN32
#include "base/strings/utf_string_conversions.h"
#endif

#include "util/misc/capture_context.h"

// Platform-specific includes for simulate crash
#if defined(__APPLE__)
  #include <TargetConditionals.h>
  #if TARGET_OS_IOS
    #include "client/simulate_crash_ios.h"
  #else
    #include "client/simulate_crash_mac.h"
  #endif
#elif defined(__linux__) || defined(__ANDROID__)
  #include "client/simulate_crash_linux.h"
#elif defined(_WIN32)
  #include "client/simulate_crash_win.h"
#endif

using namespace crashpad;

extern "C" {

// Opaque handle for CrashpadClient
typedef void* crashpad_client_t;

crashpad_client_t crashpad_client_new() {
    return new CrashpadClient();
}

void crashpad_client_delete(crashpad_client_t client) {
    delete static_cast<CrashpadClient*>(client);
}

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
    size_t extra_arguments_count) {
    
    auto* crashpad_client = static_cast<CrashpadClient*>(client);
    
#ifdef _WIN32
    // Windows uses wide strings for paths
    base::FilePath handler(base::UTF8ToWide(handler_path));
    base::FilePath database(base::UTF8ToWide(database_path));
    base::FilePath metrics(base::UTF8ToWide(metrics_path));
#else
    base::FilePath handler(handler_path);
    base::FilePath database(database_path);
    base::FilePath metrics(metrics_path);
#endif
    
    std::string url_str(url ? url : "");
    
    std::map<std::string, std::string> annotations;
    for (size_t i = 0; i < annotations_count; i++) {
        annotations[annotations_keys[i]] = annotations_values[i];
    }
    
    std::vector<std::string> arguments;
    
    // Add extra arguments from caller
    if (extra_arguments != nullptr) {
        for (size_t i = 0; i < extra_arguments_count; i++) {
            if (extra_arguments[i]) {
                arguments.push_back(extra_arguments[i]);
            }
        }
    }
    
    bool restartable = true;
    // Linux doesn't support asynchronous start
    #ifdef __linux__
    bool asynchronous_start = false;
    #else
    bool asynchronous_start = true;  // Start asynchronously on other platforms
    #endif
    
    return crashpad_client->StartHandler(
        handler,
        database,
        metrics,
        url_str,
        annotations,
        arguments,
        restartable,
        asynchronous_start
    );
}

#ifdef _WIN32
bool crashpad_client_set_handler_ipc_pipe(
    crashpad_client_t client,
    const wchar_t* ipc_pipe) {
    
    auto* crashpad_client = static_cast<CrashpadClient*>(client);
    return crashpad_client->SetHandlerIPCPipe(ipc_pipe);
}
#endif

#if defined(__APPLE__)
bool crashpad_client_set_handler_mach_service(
    crashpad_client_t client,
    const char* service_name) {
    
    auto* crashpad_client = static_cast<CrashpadClient*>(client);
    return crashpad_client->SetHandlerMachService(service_name);
}

bool crashpad_client_use_system_default_handler(
    crashpad_client_t client) {
    
    auto* crashpad_client = static_cast<CrashpadClient*>(client);
    crashpad_client->UseSystemDefaultHandler();
    return true;  // This method returns void in Crashpad
}
#endif

#if defined(__APPLE__) && defined(TARGET_OS_IOS) && TARGET_OS_IOS
bool crashpad_client_start_in_process_handler(
    crashpad_client_t client,
    const char* database_path,
    const char* url,
    const char** annotations_keys,
    const char** annotations_values,
    size_t annotations_count) {
    
    auto* crashpad_client = static_cast<CrashpadClient*>(client);
    
    // iOS doesn't run on Windows, so no need for Windows-specific path handling
    base::FilePath database(database_path);
    std::string url_str(url ? url : "");
    
    std::map<std::string, std::string> annotations;
    for (size_t i = 0; i < annotations_count; i++) {
        annotations[annotations_keys[i]] = annotations_values[i];
    }
    
    // Empty callback for now
    CrashpadClient::ProcessPendingReportsObservationCallback callback;
    
    return CrashpadClient::StartCrashpadInProcessHandler(
        database,
        url_str,
        annotations,
        callback
    );
}

void crashpad_client_process_intermediate_dumps() {
    CrashpadClient::ProcessIntermediateDumps();
}

void crashpad_client_start_processing_pending_reports() {
    CrashpadClient::StartProcessingPendingReports();
}
#endif

// DumpWithoutCrash/SimulateCrash support
// Note: DumpWithoutCrash is only available on Windows, Linux/Android, and iOS
// On macOS, we use SimulateCrash instead
void crashpad_dump_without_crash() {
#ifdef _WIN32
    // Windows has DumpWithoutCrash
    CONTEXT context;
    CaptureContext(&context);
    CrashpadClient::DumpWithoutCrash(context);
#elif defined(__APPLE__)
  #if TARGET_OS_IOS
    // iOS has DumpWithoutCrash
    NativeCPUContext context;
    CaptureContext(&context);
    CrashpadClient::DumpWithoutCrash(&context);
  #else
    // macOS uses SimulateCrash instead of DumpWithoutCrash
    NativeCPUContext context;
    CaptureContext(&context);
    SimulateCrash(context);
  #endif
#elif defined(__linux__) || defined(__ANDROID__)
    // Linux and Android have DumpWithoutCrash
    NativeCPUContext context;
    CaptureContext(&context);
    CrashpadClient::DumpWithoutCrash(&context);
#else
    #error "Unsupported platform for dump without crash"
#endif
}

// Alternative that allows passing a pre-captured context
#ifdef _WIN32
void crashpad_dump_without_crash_with_context(const void* context) {
    const CONTEXT* ctx = static_cast<const CONTEXT*>(context);
    CrashpadClient::DumpWithoutCrash(*ctx);
}
#elif defined(__APPLE__)
  #if TARGET_OS_IOS
void crashpad_dump_without_crash_with_context(void* context) {
    NativeCPUContext* ctx = static_cast<NativeCPUContext*>(context);
    CrashpadClient::DumpWithoutCrash(ctx);
}
  #else
void crashpad_dump_without_crash_with_context(void* context) {
    NativeCPUContext* ctx = static_cast<NativeCPUContext*>(context);
    SimulateCrash(*ctx);
}
  #endif
#elif defined(__linux__) || defined(__ANDROID__)
void crashpad_dump_without_crash_with_context(void* context) {
    NativeCPUContext* ctx = static_cast<NativeCPUContext*>(context);
    CrashpadClient::DumpWithoutCrash(ctx);
}
#else
    #error "Unsupported platform for dump without crash"
#endif

} // extern "C"
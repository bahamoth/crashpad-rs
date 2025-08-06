#include "client/crashpad_client.h"
#include <memory>

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
    size_t annotations_count) {
    
    auto* crashpad_client = static_cast<CrashpadClient*>(client);
    
    base::FilePath handler(handler_path);
    base::FilePath database(database_path);
    base::FilePath metrics(metrics_path);
    
    std::string url_str(url ? url : "");
    
    std::map<std::string, std::string> annotations;
    for (size_t i = 0; i < annotations_count; i++) {
        annotations[annotations_keys[i]] = annotations_values[i];
    }
    
    std::vector<std::string> arguments;
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

} // extern "C"
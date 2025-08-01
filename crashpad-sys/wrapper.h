// Wrapper header for bindgen to generate Rust FFI bindings

#ifndef CRASHPAD_WRAPPER_H
#define CRASHPAD_WRAPPER_H

// Include main Crashpad client headers
#include "client/crashpad_client.h"
#include "client/settings.h"
#include "client/crash_report_database.h"
#include "client/crashpad_info.h"

// Include annotation headers
#include "client/annotation.h"
#include "client/annotation_list.h"

// Include handler headers for configuration
#include "handler/handler_main.h"

// Platform-specific includes
#ifdef OS_MACOSX
#include "client/simulate_crash_mac.h"
#endif

#ifdef OS_WIN
#include "client/crashpad_info.h"
#endif

// Minidump related headers
#include "minidump/minidump_file_writer.h"

#endif // CRASHPAD_WRAPPER_H
use std::env;
use std::path::PathBuf;
use std::process::Command;

/// Build script for the crashpad-sys crate.
///
/// This script handles the compilation of Google Crashpad and generates Rust bindings.
/// It manages all build dependencies including depot_tools and uses the Chromium build system.
///
/// # Environment Variables
///
/// # Environment Variables Read
///
/// - `ANDROID_NDK_HOME`: Required for Android builds. Must point to Android NDK installation.
///
/// - `CARGO_CFG_TARGET_OS`: Target operating system (set by Cargo).
///
/// - `CARGO_CFG_TARGET_ARCH`: Target architecture (set by Cargo).
///
/// - `CARGO_CFG_TARGET_ENV`: Target environment, e.g., "msvc" for Windows (set by Cargo).
///
/// # Build Process
///
/// 1. Clones depot_tools if not present
/// 2. Sets up Crashpad with gclient if not present
/// 3. Configures platform-specific build with gn
/// 4. Builds Crashpad with ninja
/// 5. Compiles wrapper.cc to provide C++ interface
/// 6. Generates Rust bindings with bindgen
/// 7. Links all necessary libraries
fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=wrapper.h");

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let workspace_root = manifest_dir.parent().unwrap();

    // depot_tools
    let depot_tools = workspace_root.join("third_party/depot_tools");
    if !depot_tools.exists() {
        println!("cargo:warning=Cloning depot_tools...");
        Command::new("git")
            .args([
                "clone",
                "https://chromium.googlesource.com/chromium/tools/depot_tools.git",
            ])
            .arg(&depot_tools)
            .status()
            .expect("Failed to clone depot_tools");
    }

    // Set PATH
    let path = format!(
        "{}:{}",
        depot_tools.display(),
        env::var("PATH").unwrap_or_default()
    );

    // Crashpad
    let crashpad_checkout = workspace_root.join("third_party/crashpad_checkout");
    let crashpad_dir = crashpad_checkout.join("crashpad");

    if !crashpad_dir.exists() {
        println!("cargo:warning=Setting up Crashpad...");
        std::fs::create_dir_all(&crashpad_checkout).unwrap();

        // .gclient
        std::fs::write(
            crashpad_checkout.join(".gclient"),
            r#"solutions = [
  {
    "name": "crashpad",
    "url": "https://chromium.googlesource.com/crashpad/crashpad.git",
    "deps_file": "DEPS",
    "managed": False,
  },
]
"#,
        )
        .unwrap();

        // git clone
        Command::new("git")
            .args([
                "clone",
                "https://chromium.googlesource.com/crashpad/crashpad.git",
            ])
            .current_dir(&crashpad_checkout)
            .status()
            .expect("Failed to clone crashpad");

        // gclient sync
        println!("cargo:warning=Running gclient sync...");
        Command::new("gclient")
            .arg("sync")
            .current_dir(&crashpad_checkout)
            .env("PATH", &path)
            .status()
            .expect("Failed to run gclient sync");
    }

    // Detect target platform
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();
    let target_arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap();
    let target_env = env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default();

    // Configure gn args
    let mut gn_args = vec!["is_debug=false".to_string()];

    match target_os.as_str() {
        "linux" => match target_arch.as_str() {
            "x86_64" => gn_args.push("target_cpu=\"x64\"".to_string()),
            "aarch64" => gn_args.push("target_cpu=\"arm64\"".to_string()),
            _ => panic!("Unsupported Linux arch: {target_arch}"),
        },
        "android" => {
            gn_args.push("target_os=\"android\"".to_string());
            match target_arch.as_str() {
                "aarch64" => gn_args.push("target_cpu=\"arm64\"".to_string()),
                "x86_64" => gn_args.push("target_cpu=\"x64\"".to_string()),
                "armv7" => gn_args.push("target_cpu=\"arm\"".to_string()),
                _ => panic!("Unsupported Android arch: {target_arch}"),
            }
            // Android NDK path
            if let Ok(ndk) = env::var("ANDROID_NDK_HOME") {
                gn_args.push(format!("android_ndk_root=\"{ndk}\""));
                gn_args.push("android_api_level=21".to_string());
            } else {
                panic!("ANDROID_NDK_HOME not set for Android build");
            }
        }
        "macos" => match target_arch.as_str() {
            "x86_64" => gn_args.push("target_cpu=\"x64\"".to_string()),
            "aarch64" => gn_args.push("target_cpu=\"arm64\"".to_string()),
            _ => panic!("Unsupported macOS arch: {target_arch}"),
        },
        "ios" => {
            gn_args.push("target_os=\"ios\"".to_string());
            match target_arch.as_str() {
                "aarch64" => gn_args.push("target_cpu=\"arm64\"".to_string()),
                "x86_64" => gn_args.push("target_cpu=\"x64\"".to_string()),
                _ => panic!("Unsupported iOS arch: {target_arch}"),
            }
        }
        "windows" => {
            gn_args.push("target_cpu=\"x64\"".to_string());
            if target_env == "msvc" {
                // MSVC configuration
            }
        }
        _ => panic!("Unsupported OS: {target_os}"),
    }

    let args_str = gn_args.join(" ");

    // Separate build directory by target
    let build_name = format!("{target_os}-{target_arch}");
    let build_dir = crashpad_dir.join("out").join(&build_name);

    // gn gen
    println!("cargo:warning=Running gn gen for {build_name}...");
    Command::new("gn")
        .args([
            "gen",
            build_dir.to_str().unwrap(),
            &format!("--args={args_str}"),
        ])
        .current_dir(&crashpad_dir)
        .env("PATH", &path)
        .status()
        .expect("Failed to run gn");

    // ninja
    println!("cargo:warning=Running ninja...");
    Command::new("ninja")
        .args(["-C", build_dir.to_str().unwrap()])
        .current_dir(&crashpad_dir)
        .env("PATH", &path)
        .status()
        .expect("Failed to run ninja");

    // Compile wrapper.cc
    println!("cargo:warning=Compiling wrapper.cc...");
    let wrapper_obj = out_dir.join("crashpad_wrapper.o");

    let mut cc_cmd = Command::new("c++");
    cc_cmd.args([
        "-c",
        "-std=c++17",
        "-I",
        crashpad_dir.to_str().unwrap(),
        "-I",
        crashpad_dir
            .join("third_party/mini_chromium/mini_chromium")
            .to_str()
            .unwrap(),
        "-o",
        wrapper_obj.to_str().unwrap(),
        manifest_dir.join("crashpad_wrapper.cc").to_str().unwrap(),
    ]);

    // Platform-specific compile flags
    match target_os.as_str() {
        "linux" => {
            cc_cmd.arg("-fPIC");
        }
        "macos" => {
            cc_cmd.args(["-fPIC", "-mmacosx-version-min=10.9"]);
            // macOS specific defines
            cc_cmd.args(["-DCRASHPAD_MACOS", "-DOS_MACOSX=1"]);
        }
        "ios" => {
            cc_cmd.args(["-fPIC", "-mios-version-min=9.0"]);
            // iOS specific defines
            cc_cmd.args(["-DCRASHPAD_IOS", "-DOS_IOS=1"]);
            if target_arch == "aarch64" {
                cc_cmd.arg("-arch").arg("arm64");
            } else if target_arch == "x86_64" {
                cc_cmd.arg("-arch").arg("x86_64");
            }
        }
        "android" => {
            if let Ok(_ndk) = env::var("ANDROID_NDK_HOME") {
                // Use NDK compiler
                // TODO: Configure NDK toolchain
            }
        }
        _ => {}
    }

    let cc_status = cc_cmd.status().expect("Failed to compile wrapper.cc");
    if !cc_status.success() {
        panic!("Failed to compile wrapper.cc: {cc_status:?}");
    }

    // Check if wrapper object file was created
    if !wrapper_obj.exists() {
        panic!("wrapper.cc compilation failed - object file not created: {wrapper_obj:?}");
    }

    // bindgen
    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Unable to generate bindings");

    bindings
        .write_to_file(out_dir.join("bindings.rs"))
        .expect("Couldn't write bindings!");

    // Linking
    let obj_dir = build_dir.join("obj");

    // Library search paths
    println!(
        "cargo:rustc-link-search=native={}",
        obj_dir.join("client").display()
    );
    println!(
        "cargo:rustc-link-search=native={}",
        obj_dir.join("util").display()
    );
    println!(
        "cargo:rustc-link-search=native={}",
        obj_dir
            .join("third_party/mini_chromium/mini_chromium/base")
            .display()
    );
    println!(
        "cargo:rustc-link-search=native={}",
        obj_dir.join("minidump").display()
    );
    println!(
        "cargo:rustc-link-search=native={}",
        obj_dir.join("snapshot").display()
    );
    println!(
        "cargo:rustc-link-search=native={}",
        obj_dir.join("handler").display()
    );
    println!("cargo:rustc-link-search=native={}", out_dir.display());

    // Create static library from wrapper object file
    let lib_path = out_dir.join("libcrashpad_wrapper.a");

    let ar_status = match target_os.as_str() {
        "macos" | "ios" => {
            // macOS/iOS uses libtool
            Command::new("libtool")
                .args([
                    "-static",
                    "-o",
                    lib_path.to_str().unwrap(),
                    wrapper_obj.to_str().unwrap(),
                ])
                .status()
                .expect("Failed to create static library with libtool")
        }
        _ => {
            // Linux and others use ar
            Command::new("ar")
                .args([
                    "rcs",
                    lib_path.to_str().unwrap(),
                    wrapper_obj.to_str().unwrap(),
                ])
                .status()
                .expect("Failed to create static library with ar")
        }
    };

    if !ar_status.success() {
        panic!("Failed to create static library: {ar_status:?}");
    }

    if !lib_path.exists() {
        panic!("Static library not created: {lib_path:?}");
    }

    // Link libraries (order matters!)
    println!("cargo:rustc-link-lib=static=crashpad_wrapper");
    println!("cargo:rustc-link-lib=static=client");
    println!("cargo:rustc-link-lib=static=common");
    println!("cargo:rustc-link-lib=static=util");
    println!("cargo:rustc-link-lib=static=mig_output"); // MIG generated code
    println!("cargo:rustc-link-lib=static=format");
    println!("cargo:rustc-link-lib=static=minidump");
    println!("cargo:rustc-link-lib=static=snapshot");
    println!("cargo:rustc-link-lib=static=context");
    println!("cargo:rustc-link-lib=static=base");

    // Pass crashpad_handler path as metadata
    let handler_name = if target_os == "windows" {
        "crashpad_handler.exe"
    } else {
        "crashpad_handler"
    };
    let handler_path = build_dir.join(handler_name);

    // Handler exists check for build verification only
    if !handler_path.exists() {
        println!(
            "cargo:warning=crashpad_handler not found at expected location: {}",
            handler_path.display()
        );
    }

    // System libraries
    match target_os.as_str() {
        "linux" => {
            println!("cargo:rustc-link-lib=stdc++");
            println!("cargo:rustc-link-lib=pthread");
        }
        "macos" => {
            println!("cargo:rustc-link-lib=c++");
            println!("cargo:rustc-link-lib=framework=Foundation");
            println!("cargo:rustc-link-lib=framework=Security");
            println!("cargo:rustc-link-lib=framework=CoreFoundation");
            println!("cargo:rustc-link-lib=framework=IOKit");
            println!("cargo:rustc-link-lib=dylib=bsm");
        }
        "ios" => {
            println!("cargo:rustc-link-lib=c++");
            println!("cargo:rustc-link-lib=framework=Foundation");
            println!("cargo:rustc-link-lib=framework=Security");
            println!("cargo:rustc-link-lib=framework=CoreFoundation");
            println!("cargo:rustc-link-lib=framework=UIKit");
        }
        "android" => {
            println!("cargo:rustc-link-lib=c++_shared");
            println!("cargo:rustc-link-lib=log");
        }
        "windows" => {
            // Windows libraries are handled by Crashpad's build
        }
        _ => {}
    }
}

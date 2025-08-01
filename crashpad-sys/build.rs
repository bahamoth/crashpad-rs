use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

fn get_depot_tools_path() -> PathBuf {
    // Try from environment variable first
    if let Ok(path) = env::var("DEPOT_TOOLS_PATH") {
        let expanded = path.replace("$HOME", &env::var("HOME").unwrap());
        return PathBuf::from(expanded);
    }
    
    // Try common locations
    let home = env::var("HOME").unwrap();
    let common_paths = [
        format!("{}/projects/depot_tools", home),
        format!("{}/depot_tools", home),
        "/opt/depot_tools".to_string(),
    ];
    
    for path in &common_paths {
        let path = PathBuf::from(path);
        if path.exists() {
            return path;
        }
    }
    
    panic!("Could not find depot_tools. Please set DEPOT_TOOLS_PATH environment variable");
}

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=wrapper.h");
    
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let workspace_root = manifest_dir.parent().unwrap();
    let crashpad_dir = workspace_root.join("third_party/crashpad");
    
    // Detect target platform
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();
    let target_arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap();
    let target_env = env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default();
    
    println!("Building Crashpad for {}-{}", target_os, target_arch);
    
    // Build Crashpad
    build_crashpad(&crashpad_dir, &out_dir, &target_os, &target_arch, &target_env);
    
    // Generate bindings
    generate_bindings(&crashpad_dir, &out_dir, &target_os);
    
    // Link libraries
    link_crashpad(&out_dir, &target_os);
}

fn build_crashpad(crashpad_dir: &Path, out_dir: &Path, target_os: &str, target_arch: &str, target_env: &str) {
    let build_dir = out_dir.join("crashpad_build");
    std::fs::create_dir_all(&build_dir).unwrap();
    
    // Configure build based on target
    match target_os {
        "macos" => build_crashpad_macos(crashpad_dir, &build_dir),
        "ios" => build_crashpad_ios(crashpad_dir, &build_dir, target_arch),
        "linux" => build_crashpad_linux(crashpad_dir, &build_dir),
        "android" => build_crashpad_android(crashpad_dir, &build_dir, target_arch),
        "windows" => build_crashpad_windows(crashpad_dir, &build_dir, target_env),
        _ => panic!("Unsupported target OS: {}", target_os),
    }
}

fn build_crashpad_macos(crashpad_dir: &Path, build_dir: &Path) {
    println!("Building Crashpad for macOS");
    
    let depot_tools = get_depot_tools_path();
    
    // Use gn to generate build files
    let status = Command::new(depot_tools.join("gn"))
        .args(&[
            "gen",
            build_dir.to_str().unwrap(),
            "--args=is_debug=false target_cpu=\"x64\" mac_deployment_target=\"10.11\""
        ])
        .current_dir(crashpad_dir)
        .status()
        .expect("Failed to run gn");
    
    if !status.success() {
        panic!("gn failed");
    }
    
    // Build with ninja
    let status = Command::new(depot_tools.join("ninja"))
        .arg("-C")
        .arg(build_dir)
        .arg("crashpad_handler")
        .arg("crashpad_client")
        .status()
        .expect("Failed to run ninja");
    
    if !status.success() {
        panic!("ninja build failed");
    }
}

fn build_crashpad_ios(crashpad_dir: &Path, build_dir: &Path, target_arch: &str) {
    println!("Building Crashpad for iOS ({})", target_arch);
    
    let depot_tools = get_depot_tools_path();
    
    let cpu = match target_arch {
        "aarch64" => "arm64",
        "x86_64" => "x64",
        _ => panic!("Unsupported iOS architecture: {}", target_arch),
    };
    
    let status = Command::new(depot_tools.join("gn"))
        .args(&[
            "gen",
            build_dir.to_str().unwrap(),
            &format!("--args=is_debug=false target_os=\"ios\" target_cpu=\"{}\" ios_deployment_target=\"12.0\"", cpu)
        ])
        .current_dir(crashpad_dir)
        .status()
        .expect("Failed to run gn");
    
    if !status.success() {
        panic!("gn failed");
    }
    
    let status = Command::new(depot_tools.join("ninja"))
        .arg("-C")
        .arg(build_dir)
        .arg("crashpad_client")
        .status()
        .expect("Failed to run ninja");
    
    if !status.success() {
        panic!("ninja build failed");
    }
}

fn build_crashpad_linux(crashpad_dir: &Path, build_dir: &Path) {
    println!("Building Crashpad for Linux");
    
    let depot_tools = get_depot_tools_path();
    
    let status = Command::new(depot_tools.join("gn"))
        .args(&[
            "gen",
            build_dir.to_str().unwrap(),
            "--args=is_debug=false target_os=\"linux\" target_cpu=\"x64\""
        ])
        .current_dir(crashpad_dir)
        .status()
        .expect("Failed to run gn");
    
    if !status.success() {
        panic!("gn failed");
    }
    
    let status = Command::new(depot_tools.join("ninja"))
        .arg("-C")
        .arg(build_dir)
        .arg("crashpad_handler")
        .arg("crashpad_client")
        .status()
        .expect("Failed to run ninja");
    
    if !status.success() {
        panic!("ninja build failed");
    }
}

fn build_crashpad_android(crashpad_dir: &Path, build_dir: &Path, target_arch: &str) {
    println!("Building Crashpad for Android ({})", target_arch);
    
    let depot_tools = get_depot_tools_path();
    
    let cpu = match target_arch {
        "aarch64" => "arm64",
        "armv7" => "arm",
        "i686" => "x86",
        "x86_64" => "x64",
        _ => panic!("Unsupported Android architecture: {}", target_arch),
    };
    
    // Android requires NDK path
    let ndk_path = env::var("ANDROID_NDK_HOME")
        .or_else(|_| env::var("ANDROID_NDK_ROOT"))
        .expect("ANDROID_NDK_HOME or ANDROID_NDK_ROOT must be set");
    
    let status = Command::new(depot_tools.join("gn"))
        .args(&[
            "gen",
            build_dir.to_str().unwrap(),
            &format!(
                "--args=is_debug=false target_os=\"android\" target_cpu=\"{}\" android_ndk_root=\"{}\" android_api_level=21",
                cpu, ndk_path
            )
        ])
        .current_dir(crashpad_dir)
        .status()
        .expect("Failed to run gn");
    
    if !status.success() {
        panic!("gn failed");
    }
    
    let status = Command::new(depot_tools.join("ninja"))
        .arg("-C")
        .arg(build_dir)
        .arg("crashpad_handler")
        .arg("crashpad_client")
        .status()
        .expect("Failed to run ninja");
    
    if !status.success() {
        panic!("ninja build failed");
    }
}

fn build_crashpad_windows(crashpad_dir: &Path, build_dir: &Path, target_env: &str) {
    println!("Building Crashpad for Windows ({})", target_env);
    
    let depot_tools = get_depot_tools_path();
    
    let status = Command::new(depot_tools.join("gn"))
        .args(&[
            "gen",
            build_dir.to_str().unwrap(),
            "--args=is_debug=false target_os=\"win\" target_cpu=\"x64\""
        ])
        .current_dir(crashpad_dir)
        .status()
        .expect("Failed to run gn");
    
    if !status.success() {
        panic!("gn failed");
    }
    
    let status = Command::new(depot_tools.join("ninja"))
        .arg("-C")
        .arg(build_dir)
        .arg("crashpad_handler")
        .arg("crashpad_client")
        .status()
        .expect("Failed to run ninja");
    
    if !status.success() {
        panic!("ninja build failed");
    }
}

fn generate_bindings(crashpad_dir: &Path, out_dir: &Path, target_os: &str) {
    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .clang_arg(format!("-I{}", crashpad_dir.display()))
        .clang_arg(format!("-I{}/third_party/mini_chromium/mini_chromium", crashpad_dir.display()))
        // Platform specific defines
        .clang_arg(match target_os {
            "macos" => "-DOS_MACOSX",
            "ios" => "-DOS_IOS",
            "linux" => "-DOS_LINUX",
            "android" => "-DOS_ANDROID",
            "windows" => "-DOS_WIN",
            _ => "",
        })
        // Parse the bindings, ignoring invalid code
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Unable to generate bindings");
    
    // Write the bindings to the $OUT_DIR/bindings.rs file
    bindings
        .write_to_file(out_dir.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}

fn link_crashpad(out_dir: &Path, target_os: &str) {
    let build_dir = out_dir.join("crashpad_build");
    
    println!("cargo:rustc-link-search=native={}", build_dir.display());
    
    // Link Crashpad libraries
    println!("cargo:rustc-link-lib=static=crashpad_client");
    println!("cargo:rustc-link-lib=static=crashpad_util");
    println!("cargo:rustc-link-lib=static=crashpad_base");
    
    // Platform-specific libraries
    match target_os {
        "macos" | "ios" => {
            println!("cargo:rustc-link-lib=framework=Foundation");
            println!("cargo:rustc-link-lib=framework=Security");
            println!("cargo:rustc-link-lib=framework=CoreFoundation");
            println!("cargo:rustc-link-lib=framework=IOKit");
            if target_os == "macos" {
                println!("cargo:rustc-link-lib=framework=ApplicationServices");
            }
        }
        "linux" | "android" => {
            println!("cargo:rustc-link-lib=pthread");
            println!("cargo:rustc-link-lib=dl");
        }
        "windows" => {
            println!("cargo:rustc-link-lib=advapi32");
            println!("cargo:rustc-link-lib=winhttp");
            println!("cargo:rustc-link-lib=version");
            println!("cargo:rustc-link-lib=powrprof");
        }
        _ => {}
    }
    
    // Link C++ standard library
    let cpp_lib = if target_os == "macos" || target_os == "ios" {
        "c++"
    } else if target_os == "windows" {
        // MSVC doesn't need explicit linking
        ""
    } else {
        "stdc++"
    };
    
    if !cpp_lib.is_empty() {
        println!("cargo:rustc-link-lib={}", cpp_lib);
    }
}
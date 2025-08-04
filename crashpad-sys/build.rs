use std::env;
use std::path::PathBuf;
use std::process::Command;

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
            .args(&["clone", "https://chromium.googlesource.com/chromium/tools/depot_tools.git"])
            .arg(&depot_tools)
            .status()
            .expect("Failed to clone depot_tools");
    }
    
    // PATH 설정
    let path = format!("{}:{}", depot_tools.display(), env::var("PATH").unwrap_or_default());
    
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
        ).unwrap();
        
        // git clone
        Command::new("git")
            .args(&["clone", "https://chromium.googlesource.com/crashpad/crashpad.git"])
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
    
    // 타겟 플랫폼 감지
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();
    let target_arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap();
    let target_env = env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default();
    
    // gn args 구성
    let mut gn_args = vec!["is_debug=false".to_string()];
    
    match target_os.as_str() {
        "linux" => {
            match target_arch.as_str() {
                "x86_64" => gn_args.push("target_cpu=\"x64\"".to_string()),
                "aarch64" => gn_args.push("target_cpu=\"arm64\"".to_string()),
                _ => panic!("Unsupported Linux arch: {}", target_arch),
            }
        }
        "android" => {
            gn_args.push("target_os=\"android\"".to_string());
            match target_arch.as_str() {
                "aarch64" => gn_args.push("target_cpu=\"arm64\"".to_string()),
                "x86_64" => gn_args.push("target_cpu=\"x64\"".to_string()),
                "armv7" => gn_args.push("target_cpu=\"arm\"".to_string()),
                _ => panic!("Unsupported Android arch: {}", target_arch),
            }
            // Android NDK 경로
            if let Ok(ndk) = env::var("ANDROID_NDK_HOME") {
                gn_args.push(format!("android_ndk_root=\"{}\"", ndk));
                gn_args.push("android_api_level=21".to_string());
            } else {
                panic!("ANDROID_NDK_HOME not set for Android build");
            }
        }
        "macos" => {
            match target_arch.as_str() {
                "x86_64" => gn_args.push("target_cpu=\"x64\"".to_string()),
                "aarch64" => gn_args.push("target_cpu=\"arm64\"".to_string()),
                _ => panic!("Unsupported macOS arch: {}", target_arch),
            }
        }
        "ios" => {
            gn_args.push("target_os=\"ios\"".to_string());
            match target_arch.as_str() {
                "aarch64" => gn_args.push("target_cpu=\"arm64\"".to_string()),
                "x86_64" => gn_args.push("target_cpu=\"x64\"".to_string()),
                _ => panic!("Unsupported iOS arch: {}", target_arch),
            }
        }
        "windows" => {
            gn_args.push("target_cpu=\"x64\"".to_string());
            if target_env == "msvc" {
                // MSVC 설정
            }
        }
        _ => panic!("Unsupported OS: {}", target_os),
    }
    
    let args_str = gn_args.join(" ");
    
    // 빌드 디렉토리를 타겟별로 분리
    let build_name = format!("{}-{}", target_os, target_arch);
    let build_dir = crashpad_dir.join("out").join(&build_name);
    
    // gn gen
    println!("cargo:warning=Running gn gen for {}...", build_name);
    Command::new("gn")
        .args(&["gen", build_dir.to_str().unwrap(), &format!("--args={}", args_str)])
        .current_dir(&crashpad_dir)
        .env("PATH", &path)
        .status()
        .expect("Failed to run gn");
    
    // ninja
    println!("cargo:warning=Running ninja...");
    Command::new("ninja")
        .args(&["-C", build_dir.to_str().unwrap()])
        .current_dir(&crashpad_dir)
        .env("PATH", &path)
        .status()
        .expect("Failed to run ninja");
    
    // wrapper.cc 컴파일
    println!("cargo:warning=Compiling wrapper.cc...");
    let wrapper_obj = out_dir.join("crashpad_wrapper.o");
    
    let mut cc_cmd = Command::new("c++");
    cc_cmd.args(&[
        "-c",
        "-std=c++17",
        "-I", crashpad_dir.to_str().unwrap(),
        "-I", crashpad_dir.join("third_party/mini_chromium/mini_chromium").to_str().unwrap(),
        "-o", wrapper_obj.to_str().unwrap(),
        manifest_dir.join("crashpad_wrapper.cc").to_str().unwrap(),
    ]);
    
    // 플랫폼별 컴파일 플래그
    match target_os.as_str() {
        "linux" => {
            cc_cmd.arg("-fPIC");
        }
        "android" => {
            if let Ok(_ndk) = env::var("ANDROID_NDK_HOME") {
                // NDK 컴파일러 사용
                // TODO: NDK toolchain 설정
            }
        }
        _ => {}
    }
    
    let cc_status = cc_cmd.status().expect("Failed to compile wrapper.cc");
    if !cc_status.success() {
        panic!("Failed to compile wrapper.cc: {:?}", cc_status);
    }
    
    // wrapper 오브젝트 파일이 생성되었는지 확인
    if !wrapper_obj.exists() {
        panic!("wrapper.cc compilation failed - object file not created: {:?}", wrapper_obj);
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
    
    // 링킹
    let obj_dir = build_dir.join("obj");
    
    // 라이브러리 검색 경로
    println!("cargo:rustc-link-search=native={}", obj_dir.join("client").display());
    println!("cargo:rustc-link-search=native={}", obj_dir.join("util").display());
    println!("cargo:rustc-link-search=native={}", obj_dir.join("third_party/mini_chromium/mini_chromium/base").display());
    println!("cargo:rustc-link-search=native={}", out_dir.display());
    
    // wrapper 오브젝트 파일을 정적 라이브러리로 만들기
    let lib_path = out_dir.join("libcrashpad_wrapper.a");
    let ar_status = Command::new("ar")
        .args(&["rcs", lib_path.to_str().unwrap(), wrapper_obj.to_str().unwrap()])
        .status()
        .expect("Failed to create static library");
    
    if !ar_status.success() {
        panic!("Failed to create static library: {:?}", ar_status);
    }
    
    if !lib_path.exists() {
        panic!("Static library not created: {:?}", lib_path);
    }
    
    // 라이브러리 링크
    println!("cargo:rustc-link-lib=static=crashpad_wrapper");
    println!("cargo:rustc-link-lib=static=client");
    println!("cargo:rustc-link-lib=static=util");
    println!("cargo:rustc-link-lib=static=base");
    
    // 시스템 라이브러리
    println!("cargo:rustc-link-lib=stdc++");
    println!("cargo:rustc-link-lib=pthread");
}
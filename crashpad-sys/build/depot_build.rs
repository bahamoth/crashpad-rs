/// depot_tools를 사용한 Crashpad 빌드
///
/// 공식 Crashpad 빌드 프로세스를 따르는 완전히 독립적인 빌드 워크플로우
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// depot_tools를 사용하여 Crashpad 빌드
pub fn build_with_depot_tools() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = PathBuf::from(env::var("OUT_DIR")?);
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?);
    let target = env::var("TARGET")?;
    let profile = env::var("PROFILE").unwrap_or_else(|_| "debug".to_string());

    // Clean previous vendored build artifacts if they exist
    eprintln!("Cleaning any previous build artifacts...");
    let build_dir = manifest_dir.join("third_party/crashpad/out");
    if build_dir.exists() {
        eprintln!("Removing vendored build directory: {}", build_dir.display());
        fs::remove_dir_all(&build_dir)?;
    }

    // 1. depot_tools 다운로드 및 설정
    let depot_tools_dir = download_depot_tools()?;

    // PATH에 depot_tools 추가
    let path = env::var("PATH").unwrap_or_default();
    #[cfg(windows)]
    let path_sep = ";";
    #[cfg(not(windows))]
    let path_sep = ":";
    env::set_var(
        "PATH",
        format!("{}{}{}", depot_tools_dir.display(), path_sep, path),
    );

    // depot_tools 자동 업데이트 비활성화
    env::set_var("DEPOT_TOOLS_UPDATE", "0");
    env::set_var("DEPOT_TOOLS_METRICS", "0");

    // depot_tools는 clone 후 바로 사용 가능
    eprintln!("depot_tools ready at: {}", depot_tools_dir.display());

    // 2. 임시 작업 디렉토리 생성
    let temp_dir = out_dir.join("depot_build");
    if temp_dir.exists() {
        fs::remove_dir_all(&temp_dir)?;
    }
    fs::create_dir_all(&temp_dir)?;

    // 3. .gclient 파일 생성
    let gclient_content = r#"solutions = [
  {
    "name": "crashpad",
    "url": "https://chromium.googlesource.com/crashpad/crashpad.git",
    "managed": True,
    "custom_deps": {},
    "custom_vars": {},
  },
]"#;
    fs::write(temp_dir.join(".gclient"), gclient_content)?;

    // 4. gclient sync 실행
    eprintln!("Running gclient sync...");
    let gclient_path = if cfg!(windows) {
        depot_tools_dir.join("gclient.bat")
    } else {
        depot_tools_dir.join("gclient")
    };
    let status = Command::new(&gclient_path)
        .arg("sync")
        .current_dir(&temp_dir)
        .env("DEPOT_TOOLS_UPDATE", "0")
        .env("DEPOT_TOOLS_METRICS", "0")
        .status()?;

    if !status.success() {
        return Err("gclient sync failed".into());
    }

    // 5. crashpad 디렉토리 확인
    let crashpad_dir = temp_dir.join("crashpad");

    // 6. GN args 준비
    let mut gn_args = vec![];
    gn_args.push(format!(
        "is_debug={}",
        if profile == "debug" { "true" } else { "false" }
    ));
    gn_args.push("crashpad_build_tests=false".to_string());

    // Windows 특별 처리
    if target.contains("windows") {
        gn_args.push("target_os=\"win\"".to_string());
        let arch = if target.contains("x86_64") {
            "x64"
        } else {
            "x86"
        };
        gn_args.push(format!("target_cpu=\"{}\"", arch));

        // CRT 설정
        let crt_flag = if profile == "debug" { "/MDd" } else { "/MD" };
        gn_args.push(format!("extra_cflags=\"{}\"", crt_flag));
    }

    // 7. GN gen 실행
    let build_dir = crashpad_dir.join("out/Default");
    eprintln!("Running gn gen...");
    let gn_path = if cfg!(windows) {
        depot_tools_dir.join("gn.bat")
    } else {
        depot_tools_dir.join("gn")
    };

    // GN args를 --args= 형식으로 전달
    let args_string = format!("--args={}", gn_args.join(" "));
    eprintln!(
        "GN command: {} gen out/Default {}",
        gn_path.display(),
        args_string
    );

    let status = Command::new(&gn_path)
        .arg("gen")
        .arg("out/Default")
        .arg(&args_string)
        .current_dir(&crashpad_dir)
        .status()?;

    if !status.success() {
        return Err("gn gen failed".into());
    }

    // 8. Ninja 빌드
    eprintln!("Running ninja build...");
    let ninja_path = if cfg!(windows) {
        depot_tools_dir.join("ninja.exe")
    } else {
        depot_tools_dir.join("ninja")
    };
    let status = Command::new(&ninja_path)
        .args(&["-C", "out/Default"])
        .current_dir(&crashpad_dir)
        .status()?;

    if !status.success() {
        return Err("ninja build failed".into());
    }

    // 9. crashpad_wrapper.cc 컴파일
    compile_wrapper(&manifest_dir, &crashpad_dir, &build_dir, &out_dir)?;

    // 10. 결과물 복사
    copy_build_artifacts(&build_dir, &out_dir)?;

    // 11. bindgen 실행 (기존 코드 재사용)
    generate_bindings(&manifest_dir, &out_dir)?;

    // 12. 링크 설정
    setup_link_flags(&out_dir, &target)?;

    eprintln!("depot_tools build completed successfully");
    Ok(())
}

/// depot_tools 다운로드
fn download_depot_tools() -> Result<PathBuf, Box<dyn std::error::Error>> {
    // 캐시 디렉토리 결정
    let cache_dir = dirs::cache_dir()
        .ok_or("Failed to determine cache directory")?
        .join("crashpad-build-tools");
    fs::create_dir_all(&cache_dir)?;

    let depot_tools_dir = cache_dir.join("depot_tools");

    // 이미 다운로드되어 있는지 확인
    if depot_tools_dir.exists() {
        // 필수 파일들이 있는지 확인
        #[cfg(windows)]
        let required_files = ["gclient.bat", "gn.bat", "ninja.exe"];
        #[cfg(not(windows))]
        let required_files = ["gclient", "gn", "ninja"];

        let all_exist = required_files
            .iter()
            .all(|file| depot_tools_dir.join(file).exists());

        if all_exist {
            eprintln!(
                "Using cached depot_tools from: {}",
                depot_tools_dir.display()
            );
            return Ok(depot_tools_dir);
        }

        // 손상된 경우 재다운로드
        eprintln!("depot_tools appears incomplete, re-downloading...");
        fs::remove_dir_all(&depot_tools_dir)?;
    }

    // Git clone으로 다운로드
    eprintln!("Downloading depot_tools...");
    let status = Command::new("git")
        .args(&[
            "clone",
            "--depth",
            "1",
            "https://chromium.googlesource.com/chromium/tools/depot_tools.git",
            depot_tools_dir.to_str().unwrap(),
        ])
        .status()?;

    if !status.success() {
        return Err("Failed to clone depot_tools".into());
    }

    eprintln!("depot_tools downloaded to: {}", depot_tools_dir.display());
    Ok(depot_tools_dir)
}

/// crashpad_wrapper.cc 컴파일
fn compile_wrapper(
    manifest_dir: &Path,
    crashpad_dir: &Path,
    build_dir: &Path,
    _out_dir: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    eprintln!("Compiling crashpad_wrapper.cc...");

    let wrapper_cc = manifest_dir.join("crashpad_wrapper.cc");

    // cc crate 사용
    let mut build = cc::Build::new();
    build
        .cpp(true)
        .file(&wrapper_cc)
        .include(crashpad_dir)
        .include(build_dir.join("gen"))
        .include(crashpad_dir.join("third_party/mini_chromium/mini_chromium"))
        .flag_if_supported("-std=c++17")
        .flag_if_supported("-Wall");

    // macOS 특별 처리
    #[cfg(target_os = "macos")]
    {
        build.flag("-mmacosx-version-min=10.9");
    }

    // 컴파일
    build.compile("crashpad_wrapper");

    eprintln!("crashpad_wrapper compiled successfully");
    Ok(())
}

/// 빌드 결과물 복사
fn copy_build_artifacts(
    build_dir: &Path,
    out_dir: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    // Static libraries
    let libs = [
        "obj/client/libclient.a",
        "obj/client/libcommon.a",
        "obj/util/libutil.a",
        "obj/util/libmig_output.a", // MIG generated code
        "obj/third_party/mini_chromium/mini_chromium/base/libbase.a",
    ];

    for lib in &libs {
        let src = build_dir.join(lib);
        if src.exists() {
            let filename = src.file_name().unwrap();
            let dst = out_dir.join(filename);
            fs::copy(&src, &dst)?;
            eprintln!("Copied: {} -> {}", src.display(), dst.display());
        }
    }

    // crashpad_handler 실행파일
    #[cfg(windows)]
    let handler_name = "crashpad_handler.exe";
    #[cfg(not(windows))]
    let handler_name = "crashpad_handler";

    let handler_src = build_dir.join(handler_name);
    if handler_src.exists() {
        let handler_dst = out_dir.join(handler_name);
        fs::copy(&handler_src, &handler_dst)?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&handler_dst)?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&handler_dst, perms)?;
        }
    }

    Ok(())
}

/// bindgen으로 FFI 바인딩 생성
fn generate_bindings(
    manifest_dir: &Path,
    out_dir: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let wrapper_h = manifest_dir.join("wrapper.h");
    let bindings_path = out_dir.join("bindings.rs");

    let bindings = bindgen::Builder::default()
        .header(wrapper_h.to_str().unwrap())
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .map_err(|e| format!("Failed to generate bindings: {}", e))?;

    bindings
        .write_to_file(&bindings_path)
        .map_err(|e| format!("Failed to write bindings: {}", e))?;

    eprintln!("Generated bindings at: {}", bindings_path.display());
    Ok(())
}

/// 링크 플래그 설정
fn setup_link_flags(out_dir: &Path, target: &str) -> Result<(), Box<dyn std::error::Error>> {
    // 라이브러리 검색 경로
    println!("cargo:rustc-link-search={}", out_dir.display());

    // Static libraries (crashpad_wrapper is handled by cc::Build)
    println!("cargo:rustc-link-lib=static=client");
    println!("cargo:rustc-link-lib=static=common");
    println!("cargo:rustc-link-lib=static=util");
    println!("cargo:rustc-link-lib=static=mig_output");
    println!("cargo:rustc-link-lib=static=base");

    // Platform-specific libraries
    if target.contains("windows") {
        println!("cargo:rustc-link-lib=advapi32");
        println!("cargo:rustc-link-lib=kernel32");
        println!("cargo:rustc-link-lib=user32");
        println!("cargo:rustc-link-lib=winmm");
    } else if target.contains("apple") {
        println!("cargo:rustc-link-lib=framework=Foundation");
        println!("cargo:rustc-link-lib=framework=Security");
        println!("cargo:rustc-link-lib=framework=CoreFoundation");
        println!("cargo:rustc-link-lib=framework=IOKit");
    } else {
        println!("cargo:rustc-link-lib=stdc++");
        println!("cargo:rustc-link-lib=pthread");
    }

    // Handler path 설정
    let handler_path = out_dir.join(if target.contains("windows") {
        "crashpad_handler.exe"
    } else {
        "crashpad_handler"
    });
    println!(
        "cargo:rustc-env=CRASHPAD_HANDLER_PATH={}",
        handler_path.display()
    );

    Ok(())
}

/// Prebuilt 바이너리 다운로드 및 링크
///
/// GitHub Releases에서 미리 빌드된 Crashpad 라이브러리를 다운로드
use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// Prebuilt 바이너리 다운로드 및 설정
pub fn download_and_link() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = PathBuf::from(env::var("OUT_DIR")?);
    let target = env::var("TARGET")?;
    let version = env::var("CARGO_PKG_VERSION")?;

    eprintln!("Downloading prebuilt binaries for {} v{}", target, version);

    // 캐시 디렉토리
    let cache_dir = dirs::cache_dir()
        .ok_or("Failed to determine cache directory")?
        .join("crashpad-build-tools")
        .join("prebuilt")
        .join(&version)
        .join(&target);

    // 이미 다운로드되어 있는지 확인
    let marker_file = cache_dir.join(".complete");
    if marker_file.exists() {
        eprintln!("Using cached prebuilt from: {}", cache_dir.display());
        copy_from_cache(&cache_dir, &out_dir)?;
    } else {
        // 다운로드
        download_prebuilt(&version, &target, &cache_dir)?;
        fs::write(&marker_file, "")?;
        copy_from_cache(&cache_dir, &out_dir)?;
    }

    // bindgen 실행 (wrapper.h 필요)
    generate_bindings(&out_dir)?;

    // 링크 설정
    setup_link_flags(&out_dir, &target)?;

    eprintln!("Prebuilt setup completed");
    Ok(())
}

/// GitHub Releases에서 다운로드
fn download_prebuilt(
    version: &str,
    target: &str,
    cache_dir: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all(cache_dir)?;

    // GitHub Release URL 구성
    // TODO: 실제 GitHub 저장소 URL로 변경 필요
    let url = format!(
        "https://github.com/OWNER/REPO/releases/download/v{}/crashpad-{}-{}.tar.gz",
        version, version, target
    );

    eprintln!("Downloading from: {}", url);

    // 다운로드
    let response = ureq::get(&url)
        .call()
        .map_err(|e| format!("Failed to download prebuilt: {}", e))?;

    // 임시 파일에 저장
    let temp_file = cache_dir.join("download.tar.gz");
    let mut file = fs::File::create(&temp_file)?;
    io::copy(&mut response.into_reader(), &mut file)?;

    // 압축 해제
    extract_archive(&temp_file, cache_dir)?;

    // 임시 파일 삭제
    fs::remove_file(temp_file)?;

    eprintln!("Downloaded and extracted to: {}", cache_dir.display());
    Ok(())
}

/// tar.gz 압축 해제
fn extract_archive(archive_path: &Path, dest_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    use std::process::Command;

    #[cfg(unix)]
    {
        let status = Command::new("tar")
            .args(&["-xzf", archive_path.to_str().unwrap()])
            .arg("-C")
            .arg(dest_dir)
            .status()?;

        if !status.success() {
            return Err("Failed to extract archive".into());
        }
    }

    #[cfg(windows)]
    {
        // Windows에서는 tar 명령이 Windows 10부터 기본 제공
        let status = Command::new("tar")
            .args(&["-xzf", archive_path.to_str().unwrap()])
            .arg("-C")
            .arg(dest_dir)
            .status();

        if status.is_err() || !status.unwrap().success() {
            // tar가 없으면 PowerShell 사용
            let ps_script = format!(
                "Expand-Archive -Path '{}' -DestinationPath '{}' -Force",
                archive_path.display(),
                dest_dir.display()
            );

            let status = Command::new("powershell")
                .args(&["-Command", &ps_script])
                .status()?;

            if !status.success() {
                return Err("Failed to extract archive".into());
            }
        }
    }

    Ok(())
}

/// 캐시에서 OUT_DIR로 복사
fn copy_from_cache(cache_dir: &Path, out_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    // lib 디렉토리의 모든 파일 복사
    let lib_dir = cache_dir.join("lib");
    if lib_dir.exists() {
        for entry in fs::read_dir(&lib_dir)? {
            let entry = entry?;
            let src = entry.path();
            let filename = src.file_name().unwrap();
            let dst = out_dir.join(filename);
            fs::copy(&src, &dst)?;
        }
    }

    // crashpad_handler 복사
    #[cfg(windows)]
    let handler_name = "crashpad_handler.exe";
    #[cfg(not(windows))]
    let handler_name = "crashpad_handler";

    let handler_src = cache_dir.join(handler_name);
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

    // wrapper.h 복사 (bindgen용)
    let wrapper_src = cache_dir.join("include").join("wrapper.h");
    if wrapper_src.exists() {
        let wrapper_dst = out_dir.join("wrapper.h");
        fs::copy(&wrapper_src, &wrapper_dst)?;
    }

    Ok(())
}

/// bindgen으로 FFI 바인딩 생성
fn generate_bindings(out_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let wrapper_h = out_dir.join("wrapper.h");
    let bindings_path = out_dir.join("bindings.rs");

    // wrapper.h가 없으면 현재 프로젝트에서 복사
    if !wrapper_h.exists() {
        let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?);
        let src_wrapper = manifest_dir.join("wrapper.h");
        if src_wrapper.exists() {
            fs::copy(&src_wrapper, &wrapper_h)?;
        }
    }

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

    // Static libraries
    println!("cargo:rustc-link-lib=static=crashpad");

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

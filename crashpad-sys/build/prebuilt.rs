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

    eprintln!("Using prebuilt binaries for {} v{}", target, version);

    // 캐시 디렉토리 - xtask build-prebuilt과 동일한 위치 사용
    let cache_dir = dirs::cache_dir()
        .ok_or("Failed to determine cache directory")?
        .join("crashpad-build-tools")
        .join("prebuilt")
        .join(&version)
        .join(&target);

    // 이미 다운로드되어 있는지 확인
    let marker_file = cache_dir.join(".crashpad-ok");
    if !marker_file.exists() {
        // 다운로드 및 압축 해제
        download_prebuilt(&version, &target, &cache_dir)?;
        fs::write(&marker_file, "")?;
    }
    
    eprintln!("Using cached prebuilt from: {}", cache_dir.display());
    
    // 캐시 디렉토리를 OUT_DIR로 직접 사용
    // bindings.rs는 이미 캐시에 있음
    let bindings_src = cache_dir.join("bindings.rs");
    let bindings_dst = out_dir.join("bindings.rs");
    if bindings_src.exists() {
        fs::copy(&bindings_src, &bindings_dst)?;
        eprintln!("Using pre-generated bindings");
    } else {
        eprintln!("Warning: bindings.rs not found in prebuilt package");
    }
    
    // 링크 설정 - 캐시 디렉토리를 직접 참조
    setup_link_flags(&cache_dir, &target)?;

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
    let url = format!(
        "https://github.com/kyunghoon/crashpad-rs/releases/download/v{}/crashpad-{}-{}.tar.gz",
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



/// 링크 플래그 설정
fn setup_link_flags(cache_dir: &Path, target: &str) -> Result<(), Box<dyn std::error::Error>> {
    // 라이브러리 검색 경로
    println!("cargo:rustc-link-search={}", cache_dir.display());

    if target.contains("windows") {
        // Add lib directory for all .lib files
        let lib_dir = cache_dir.join("lib");
        if lib_dir.exists() {
            println!("cargo:rustc-link-search={}", lib_dir.display());
        }
        
        // Link all necessary libraries in dependency order
        println!("cargo:rustc-link-lib=static=crashpad_wrapper");
        println!("cargo:rustc-link-lib=static=client");
        println!("cargo:rustc-link-lib=static=common");
        println!("cargo:rustc-link-lib=static=util");
        println!("cargo:rustc-link-lib=static=base");
        println!("cargo:rustc-link-lib=static=snapshot");
        println!("cargo:rustc-link-lib=static=minidump");
        println!("cargo:rustc-link-lib=static=format");
        println!("cargo:rustc-link-lib=static=handler");
        println!("cargo:rustc-link-lib=static=handler_common");
        println!("cargo:rustc-link-lib=static=context");
        println!("cargo:rustc-link-lib=static=compat");
        println!("cargo:rustc-link-lib=static=net");
        println!("cargo:rustc-link-lib=static=getopt");
        println!("cargo:rustc-link-lib=static=zlib");
    } else {
        println!("cargo:rustc-link-lib=static=crashpad");
    }

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
    let handler_path = cache_dir.join(if target.contains("windows") {
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

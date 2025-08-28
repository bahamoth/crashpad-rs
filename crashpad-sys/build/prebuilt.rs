/// Download and link prebuilt binaries
///
/// Download pre-built Crashpad libraries from GitHub Releases
use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// Download and setup prebuilt binaries
pub fn download_and_link() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = PathBuf::from(env::var("OUT_DIR")?);
    let target = env::var("TARGET")?;
    let version = env::var("CARGO_PKG_VERSION")?;

    println!(
        "cargo:warning=Using prebuilt binaries for {} v{}",
        target, version
    );

    // Cache directory
    let cache_dir = crate::cache::prebuilt_dir(&version, &target);
    println!("cargo:warning=Cache dir: {}", cache_dir.display());

    let marker_file = cache_dir.join(".crashpad-ok");
    if !marker_file.exists() {
        println!("cargo:warning=No marker file, attempting download...");
        download_prebuilt(&version, &target, &cache_dir)?;
        fs::write(&marker_file, "")?;
    }

    println!(
        "cargo:warning=Using cached prebuilt from: {}",
        cache_dir.display()
    );

    // Copy bindings.rs from cache
    let bindings_src = cache_dir.join("bindings.rs");
    let bindings_dst = out_dir.join("bindings.rs");
    if bindings_src.exists() {
        fs::copy(&bindings_src, &bindings_dst)?;
        eprintln!("Using pre-generated bindings");
    } else {
        eprintln!("Warning: bindings.rs not found in prebuilt package");
    }

    setup_link_flags(&cache_dir, &target)?;

    // Copy handler to target directory for distribution
    copy_handler_to_target(&cache_dir, &target)?;

    eprintln!("Prebuilt setup completed");
    Ok(())
}

/// Download from GitHub Releases
fn download_prebuilt(
    version: &str,
    target: &str,
    cache_dir: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all(cache_dir)?;

    let url = format!(
        "https://github.com/bahamoth/crashpad-rs/releases/download/v{}/crashpad-{}-{}.tar.gz",
        version, version, target
    );

    println!("cargo:warning=Downloading from: {}", url);

    let response = ureq::get(&url).call().map_err(|e| {
        println!(
            "cargo:warning=Note: Prebuilt binaries not available at {}",
            url
        );
        println!("cargo:warning=This is expected if releases haven't been published yet");
        format!("Failed to download prebuilt: {}", e)
    })?;

    let temp_file = cache_dir.join("download.tar.gz");
    let mut file = fs::File::create(&temp_file)?;
    io::copy(&mut response.into_reader(), &mut file)?;

    extract_archive(&temp_file, cache_dir)?;

    fs::remove_file(temp_file)?;

    eprintln!("Downloaded and extracted to: {}", cache_dir.display());
    Ok(())
}

/// Extract tar.gz archive
fn extract_archive(archive_path: &Path, dest_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    use std::process::Command;

    #[cfg(unix)]
    {
        let status = Command::new("tar")
            .args(["-xzf", archive_path.to_str().unwrap()])
            .arg("-C")
            .arg(dest_dir)
            .status()?;

        if !status.success() {
            return Err("Failed to extract archive".into());
        }
    }

    #[cfg(windows)]
    {
        // Windows 10+ includes tar command
        let status = Command::new("tar")
            .args(["-xzf", archive_path.to_str().unwrap()])
            .arg("-C")
            .arg(dest_dir)
            .status();

        if status.is_err() || !status.unwrap().success() {
            // Fall back to PowerShell if tar is unavailable
            let ps_script = format!(
                "Expand-Archive -Path '{}' -DestinationPath '{}' -Force",
                archive_path.display(),
                dest_dir.display()
            );

            let status = Command::new("powershell")
                .args(["-Command", &ps_script])
                .status()?;

            if !status.success() {
                return Err("Failed to extract archive".into());
            }
        }
    }

    Ok(())
}

/// Setup link flags
fn setup_link_flags(cache_dir: &Path, target: &str) -> Result<(), Box<dyn std::error::Error>> {
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
    } else if target.contains("apple") {
        // macOS and iOS need wrapper plus actual libraries
        println!("cargo:rustc-link-lib=static=crashpad_wrapper");
        println!("cargo:rustc-link-lib=static=client");
        println!("cargo:rustc-link-lib=static=common");
        println!("cargo:rustc-link-lib=static=util");
        println!("cargo:rustc-link-lib=static=format");
        println!("cargo:rustc-link-lib=static=base");
        println!("cargo:rustc-link-lib=static=mig_output");

        // iOS-specific libraries for in-process handler
        if target.contains("ios") {
            println!("cargo:rustc-link-lib=static=snapshot");
            println!("cargo:rustc-link-lib=static=context");
            println!("cargo:rustc-link-lib=static=minidump");
        }
    } else {
        // Linux/Android
        println!("cargo:rustc-link-lib=static=crashpad_wrapper");
        println!("cargo:rustc-link-lib=static=client");
        println!("cargo:rustc-link-lib=static=common");
        println!("cargo:rustc-link-lib=static=util");
        println!("cargo:rustc-link-lib=static=format");
        println!("cargo:rustc-link-lib=static=base");
    }

    // Platform-specific libraries
    if target.contains("windows") {
        println!("cargo:rustc-link-lib=advapi32");
        println!("cargo:rustc-link-lib=kernel32");
        println!("cargo:rustc-link-lib=user32");
        println!("cargo:rustc-link-lib=winmm");
    } else if target.contains("apple-ios") {
        println!("cargo:rustc-link-lib=framework=Foundation");
        println!("cargo:rustc-link-lib=framework=Security");
        println!("cargo:rustc-link-lib=framework=CoreFoundation");
        println!("cargo:rustc-link-lib=framework=UIKit");
        println!("cargo:rustc-link-lib=c++");
        println!("cargo:rustc-link-lib=z");
    } else if target.contains("apple-darwin") {
        println!("cargo:rustc-link-lib=framework=Foundation");
        println!("cargo:rustc-link-lib=framework=Security");
        println!("cargo:rustc-link-lib=framework=CoreFoundation");
        println!("cargo:rustc-link-lib=framework=IOKit");
        println!("cargo:rustc-link-lib=dylib=bsm");
        println!("cargo:rustc-link-lib=c++");
    } else if target.contains("android") {
        // Android uses libc++ instead of libstdc++
        println!("cargo:rustc-link-lib=c++_static");
        println!("cargo:rustc-link-lib=c++abi");
    } else {
        println!("cargo:rustc-link-lib=stdc++");
        println!("cargo:rustc-link-lib=pthread");
    }

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

/// Copy crashpad_handler to target directory for distribution
fn copy_handler_to_target(
    cache_dir: &Path,
    target: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // iOS doesn't have external handler
    if target.contains("ios") {
        return Ok(());
    }

    let handler_name = if target.contains("windows") {
        "crashpad_handler.exe"
    } else if target.contains("android") {
        "libcrashpad_handler.so"
    } else {
        "crashpad_handler"
    };

    let handler_src = cache_dir.join(handler_name);

    // Skip if handler doesn't exist
    if !handler_src.exists() {
        eprintln!("Warning: Handler not found at {}", handler_src.display());
        return Ok(());
    }

    // Determine target directory honoring CARGO_TARGET_DIR and Cargo layout
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?);
    let profile = env::var("PROFILE").unwrap_or_else(|_| "debug".to_string());
    let host = env::var("HOST").unwrap_or_else(|_| target.to_string());
    let is_cross_compile = host != target;

    let root = if let Ok(dir) = env::var("CARGO_TARGET_DIR") {
        PathBuf::from(dir)
    } else if let Ok(out) = env::var("OUT_DIR") {
        let mut p = PathBuf::from(out);
        for _ in 0..5 {
            if p.file_name().map(|s| s == "target").unwrap_or(false) {
                break;
            }
            if !p.pop() {
                break;
            }
        }
        if p.file_name().map(|s| s == "target").unwrap_or(false) {
            p
        } else {
            manifest_dir
                .parent()
                .ok_or("Failed to get parent directory")?
                .join("target")
        }
    } else {
        manifest_dir
            .parent()
            .ok_or("Failed to get parent directory")?
            .join("target")
    };

    let target_dir = if is_cross_compile {
        root.join(target).join(&profile)
    } else {
        root.join(&profile)
    };

    fs::create_dir_all(&target_dir)?;

    let handler_dest = target_dir.join(handler_name);

    eprintln!(
        "Copying handler from {} to {}",
        handler_src.display(),
        handler_dest.display()
    );
    fs::copy(&handler_src, &handler_dest)?;

    // Set executable permissions on Unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&handler_dest)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&handler_dest, perms)?;
    }

    // Expose handler path to dependents via DEP_<links>_HANDLER
    println!("cargo:handler={}", handler_dest.display());
    eprintln!("Handler copied to target directory");
    Ok(())
}

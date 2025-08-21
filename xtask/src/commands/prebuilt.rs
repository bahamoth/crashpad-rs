use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use xshell::{cmd, Shell};

use crate::utils::find_workspace_root;

struct BuildArtifacts {
    out_dir: PathBuf,  // The entire OUT_DIR from vendored-depot build
}

/// Build prebuilt packages for distribution
pub fn build_prebuilt(sh: &Shell, target: Option<String>) -> Result<()> {
    println!("🔨 Building prebuilt package...");

    let workspace_root = find_workspace_root(sh)?;
    sh.change_dir(&workspace_root);

    // Get target triple
    let target = target.unwrap_or_else(|| {
        std::env::var("TARGET").unwrap_or_else(|_| {
            // Detect current platform
            let output = std::process::Command::new("rustc")
                .args(&["-vV"])
                .output()
                .expect("Failed to get rustc version");
            let output_str = String::from_utf8_lossy(&output.stdout);

            // Extract host triple from rustc output
            for line in output_str.lines() {
                if line.starts_with("host:") {
                    return line.split_whitespace().nth(1).unwrap().to_string();
                }
            }

            panic!("Could not determine target triple");
        })
    });

    println!("📦 Target: {}", target);

    // Build crashpad using appropriate feature for platform
    println!("🏗️  Building crashpad...");
    let feature = if target.contains("windows") {
        "vendored-depot"
    } else {
        "vendored"
    };
    
    cmd!(sh, "cargo build --package crashpad-rs-sys --release --no-default-features --features {feature} --target {target}").run()?;

    // Get package version
    let version = get_package_version(&workspace_root)?;
    println!("📌 Version: {}", version);

    // Find the OUT_DIR from vendored-depot build
    println!("📂 Finding build output directory...");
    let out_dir = find_build_output_dir(&workspace_root, &target)?;
    println!("  Found OUT_DIR: {}", out_dir.display());
    
    let artifacts = BuildArtifacts { out_dir };
    
    // Create prebuilt directory structure in target/ first
    let prebuilt_dir = workspace_root
        .join("target")
        .join(&target)
        .join("crashpad-prebuilt")
        .join(&version);
    
    // Clean and create directories
    if prebuilt_dir.exists() {
        sh.remove_path(&prebuilt_dir)?;
    }
    sh.create_dir(&prebuilt_dir)?;

    // Copy only necessary files
    println!("📚 Collecting build artifacts...");
    
    // 1. Copy bindings.rs
    let bindings_src = artifacts.out_dir.join("bindings.rs");
    if bindings_src.exists() {
        let bindings_dest = prebuilt_dir.join("bindings.rs");
        fs::copy(&bindings_src, &bindings_dest)?;
        println!("  ✓ bindings.rs");
    }
    
    // 2. Copy crashpad_wrapper.lib
    let wrapper_lib_src = artifacts.out_dir.join("crashpad_wrapper.lib");
    if wrapper_lib_src.exists() {
        let wrapper_lib_dest = prebuilt_dir.join("crashpad_wrapper.lib");
        fs::copy(&wrapper_lib_src, &wrapper_lib_dest)?;
        println!("  ✓ crashpad_wrapper.lib");
    }
    
    if target.contains("windows") {
        // 3. Copy all .lib files from crashpad_build/obj/
        let crashpad_build_dir = workspace_root
            .join(format!("target/{}/release/crashpad_build", target));
        
        if crashpad_build_dir.exists() {
            // Create lib directory for organization
            let lib_dir = prebuilt_dir.join("lib");
            sh.create_dir(&lib_dir)?;
            
            // List of required .lib files with their paths
            let lib_files = vec![
                ("obj/client/client.lib", "client.lib"),
                ("obj/client/common.lib", "common.lib"),
                ("obj/util/util.lib", "util.lib"),
                ("obj/third_party/mini_chromium/mini_chromium/base/base.lib", "base.lib"),
                ("obj/third_party/zlib/zlib.lib", "zlib.lib"),
                ("obj/snapshot/context.lib", "context.lib"),
                ("obj/snapshot/snapshot.lib", "snapshot.lib"),
                ("obj/minidump/format.lib", "format.lib"),
                ("obj/minidump/minidump.lib", "minidump.lib"),
                ("obj/handler/handler.lib", "handler.lib"),
                ("obj/handler/common.lib", "handler_common.lib"),
                ("obj/compat/compat.lib", "compat.lib"),
                ("obj/util/net.lib", "net.lib"),
                ("obj/third_party/getopt/getopt.lib", "getopt.lib"),
            ];
            
            for (src_path, dest_name) in lib_files {
                let src = crashpad_build_dir.join(src_path);
                if src.exists() {
                    let dest = lib_dir.join(dest_name);
                    fs::copy(&src, &dest)?;
                    println!("  ✓ {}", dest_name);
                } else {
                    println!("  ⚠ Missing: {}", src_path);
                }
            }
        }
        
        // 4. Copy crashpad_handler.exe
        let handler_src = crashpad_build_dir.join("crashpad_handler.exe");
        if handler_src.exists() {
            let handler_dest = prebuilt_dir.join("crashpad_handler.exe");
            fs::copy(&handler_src, &handler_dest)?;
            println!("  ✓ crashpad_handler.exe");
        } else {
            println!("  ⚠ crashpad_handler.exe not found at {}", handler_src.display());
        }
    }
    
    // Don't create marker here - it will be created after extraction in cache

    // Create distribution archive for GitHub releases
    // Place under target/ for easy cleanup with cargo clean
    let archive_dir = workspace_root.join("target").join("prebuilt-archives");
    sh.create_dir(&archive_dir)?;
    
    let archive_name = format!("crashpad-{}-{}.tar.gz", version, target);
    let archive_path = archive_dir.join(&archive_name);
    
    println!("📦 Creating archive: {}", archive_name);
    
    // Create tar archive of contents (not the directory itself)
    // Use . to include all files in the directory without creating a parent folder
    cmd!(sh, "tar -czf {archive_path} -C {prebuilt_dir} .").run()?;

    // Generate checksum
    println!("🔐 Generating checksum...");
    let archive_content = fs::read(&archive_path)?;
    let digest = sha256::digest(&archive_content[..]);
    let checksum_path = archive_path.with_extension("tar.gz.sha256");
    fs::write(&checksum_path, format!("{}  {}\n", digest, archive_name))?;

    // Simulate GitHub download by copying to cache and extracting
    println!("\n📥 Simulating GitHub download to cache...");
    let cache_dir = dirs::cache_dir()
        .context("Failed to determine cache directory")?
        .join("crashpad-build-tools")
        .join("prebuilt")
        .join(&version)
        .join(&target);
    
    // Clean and create cache directory
    if cache_dir.exists() {
        sh.remove_path(&cache_dir)?;
    }
    sh.create_dir(&cache_dir)?;
    
    // Copy archive to cache (simulating download)
    let cache_archive = cache_dir.join(&archive_name);
    fs::copy(&archive_path, &cache_archive)?;
    println!("  ✓ Copied archive to cache");
    
    // Extract in cache (same as prebuilt.rs would do)
    cmd!(sh, "tar -xzf {cache_archive} -C {cache_dir}").run()?;
    println!("  ✓ Extracted in cache");
    
    // Create marker file
    let marker_file = cache_dir.join(".crashpad-ok");
    fs::write(&marker_file, "")?;
    println!("  ✓ Created .crashpad-ok marker");
    
    // Clean up the archive from cache (it's already in target/prebuilt-archives/)
    fs::remove_file(&cache_archive)?;
    
    println!("\n✅ Prebuilt package created:");
    println!("  📁 Build: {}", prebuilt_dir.display());
    println!("  📁 Cache: {}", cache_dir.display());
    println!("  📦 Archive: {}", archive_path.display());
    println!("  🔐 Checksum: {}", checksum_path.display());
    println!("\n📤 Ready to upload to GitHub Releases!");
    println!("\n🧪 Test locally with: cargo build --package crashpad-rs-sys --features prebuilt");

    Ok(())
}

/// Recursively copy directory
fn copy_dir_all(src: &Path, dst: &Path) -> Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let path = entry.path();
        let file_name = path.file_name().unwrap();
        let dest = dst.join(file_name);
        
        if path.is_dir() {
            copy_dir_all(&path, &dest)?;
        } else {
            fs::copy(&path, &dest)?;
        }
    }
    Ok(())
}

/// Get package version from Cargo.toml
fn get_package_version(workspace_root: &Path) -> Result<String> {
    
    // Parse version from workspace inheritance
    let workspace_toml = workspace_root.join("Cargo.toml");
    let workspace_content = fs::read_to_string(&workspace_toml)?;
    
    let version = workspace_content
        .lines()
        .skip_while(|line| !line.starts_with("[workspace.package]"))
        .find(|line| line.starts_with("version"))
        .and_then(|line| line.split('=').nth(1))
        .map(|v| v.trim().trim_matches('"'))
        .context("Failed to parse version")?
        .to_string();
    
    Ok(version)
}


/// Find the build output directory containing the libraries
fn find_build_output_dir(workspace_root: &Path, target: &str) -> Result<PathBuf> {
    // Common patterns for build output
    let candidates = vec![
        workspace_root.join(format!("target/{}/release/build", target)),
        workspace_root.join("target/release/build"),
    ];
    
    for candidate in candidates {
        if candidate.exists() {
            // Find crashpad-rs-sys build directory
            for entry in fs::read_dir(&candidate)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() && path.to_string_lossy().contains("crashpad-rs-sys") {
                    let out_dir = path.join("out");
                    if out_dir.exists() {
                        return Ok(out_dir);
                    }
                }
            }
        }
    }
    
    anyhow::bail!(
        "Could not find build output directory for target {}. Make sure the build completed successfully.",
        target
    )
}
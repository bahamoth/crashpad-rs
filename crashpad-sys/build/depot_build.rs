#![allow(dead_code)]

/// Build Crashpad using depot_tools
///
/// Uses official Crashpad build process for all platforms including Windows
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::tools::{depot_cmd, ensure_depot_tools, setup_depot_tools_env};

/// Main entry point for build.rs
pub fn build_with_depot_tools() -> Result<(), Box<dyn std::error::Error>> {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?);
    let target = env::var("TARGET")?;
    let profile = env::var("PROFILE").unwrap_or_else(|_| "debug".to_string());

    // Step 1: Setup depot_tools
    let platform_dir = manifest_dir
        .parent()
        .expect("Failed to get parent directory")
        .join("target")
        .join(&target);
    let depot_tools_dir = ensure_depot_tools(&platform_dir)?;
    setup_depot_tools_env(&depot_tools_dir)?;

    // Step 2: Build Crashpad with depot_tools
    // Use a permanent location for crashpad source, not a temp directory
    let build_dir = platform_dir.join("crashpad_source");
    let build_output = build_crashpad_with_depot(
        &depot_tools_dir,
        &build_dir,
        &manifest_dir,
        &target,
        &profile,
    )?;

    // Step 3: Build crashpad-rs-sys
    build_crashpad_sys(&build_output, &target)?;

    Ok(())
}

/// Structure to hold Crashpad build output paths
pub struct CrashpadBuildOutput {
    pub crashpad_dir: PathBuf,
    pub build_out_dir: PathBuf,
}

/// Build Crashpad using depot_tools
pub fn build_crashpad_with_depot(
    depot_tools_dir: &Path,
    build_dir: &Path,
    manifest_dir: &Path,
    target: &str,
    profile: &str,
) -> Result<CrashpadBuildOutput, Box<dyn std::error::Error>> {
    // Configure output directory to be in target/{target}/{profile}/crashpad_build
    let final_build_dir = manifest_dir
        .parent()
        .expect("Failed to get parent directory")
        .join("target")
        .join(target)
        .join(profile)
        .join("crashpad_build");

    // Check for build completion marker
    let marker_file = final_build_dir.join(".crashpad-ok");
    if marker_file.exists() {
        println!("cargo:warning=Using cached Crashpad build (.crashpad-ok found)");
        println!(
            "cargo:warning=Note: If crashpad_wrapper.cc was modified, delete {} and rebuild",
            marker_file.display()
        );

        // For cached builds, crashpad source should be at permanent location
        let crashpad_dir = build_dir.join("crashpad");

        // If source doesn't exist, we need to sync it
        if !crashpad_dir.exists() {
            // Silently sync source if needed

            // Create work directory
            fs::create_dir_all(build_dir)?;

            // Create .gclient file
            let gclient_content = r#"solutions = [
  {
    "name": "crashpad",
    "url": "https://chromium.googlesource.com/crashpad/crashpad.git",
    "managed": True,
    "custom_deps": {},
    "custom_vars": {},
  },
]"#;
            fs::write(build_dir.join(".gclient"), gclient_content)?;

            // Run gclient sync to get source
            let gclient = depot_cmd(depot_tools_dir, "gclient");
            let status = Command::new(&gclient)
                .args(["sync", "--no-history"])
                .current_dir(build_dir)
                .env("DEPOT_TOOLS_WIN_TOOLCHAIN", "0")
                .status()?;

            if !status.success() {
                return Err("gclient sync failed".into());
            }

            // Copy crashpad_wrapper.cc
            fs::copy(
                manifest_dir.join("crashpad_wrapper.cc"),
                crashpad_dir.join("crashpad_wrapper.cc"),
            )?;
        }

        return Ok(CrashpadBuildOutput {
            build_out_dir: final_build_dir,
            crashpad_dir,
        });
    }

    // Don't clean if .gclient already exists (source is already there)
    if !build_dir.join(".gclient").exists() {
        // Clean and create work directory only if no existing source
        if build_dir.exists() {
            fs::remove_dir_all(build_dir)?;
        }
        fs::create_dir_all(build_dir)?;
    }

    // Only create .gclient and sync if not already done
    if !build_dir.join(".gclient").exists() {
        // Create .gclient file
        let gclient_content = r#"solutions = [
  {
    "name": "crashpad",
    "url": "https://chromium.googlesource.com/crashpad/crashpad.git",
    "managed": True,
    "custom_deps": {},
    "custom_vars": {},
  },
]"#;
        fs::write(build_dir.join(".gclient"), gclient_content)?;

        // Run gclient sync
        let gclient = depot_cmd(depot_tools_dir, "gclient");
        let status = Command::new(&gclient)
            .args(["sync", "--no-history"])
            .current_dir(build_dir)
            .env("DEPOT_TOOLS_WIN_TOOLCHAIN", "0")
            .status()?;

        if !status.success() {
            return Err("gclient sync failed".into());
        }
    }

    let crashpad_dir = build_dir.join("crashpad");

    // Copy crashpad_wrapper.cc (always copy to ensure it's up to date)
    fs::copy(
        manifest_dir.join("crashpad_wrapper.cc"),
        crashpad_dir.join("crashpad_wrapper.cc"),
    )?;

    // Configure GN build args
    let mut gn_args = vec![
        format!(
            "is_debug={}",
            if profile == "debug" { "true" } else { "false" }
        ),
        "crashpad_build_tests=false".to_string(),
    ];

    if target.contains("windows") {
        gn_args.push("target_os=\"win\"".to_string());
        gn_args.push(format!(
            "target_cpu=\"{}\"",
            if target.contains("x86_64") {
                "x64"
            } else {
                "x86"
            }
        ));
        gn_args.push(format!(
            "extra_cflags=\"{}\"",
            if profile == "debug" { "/MDd" } else { "/MD" }
        ));
    }

    // Create the output directory if it doesn't exist
    fs::create_dir_all(&final_build_dir)?;

    // Run GN gen with absolute output path
    let gn = depot_cmd(depot_tools_dir, "gn");
    let status = Command::new(&gn)
        .args([
            "gen",
            final_build_dir.to_str().unwrap(),
            &format!("--args={}", gn_args.join(" ")),
        ])
        .current_dir(&crashpad_dir)
        .status()?;

    if !status.success() {
        return Err("gn gen failed".into());
    }

    // Run Ninja build - explicitly build library targets
    let ninja = depot_cmd(depot_tools_dir, "ninja");
    let status = Command::new(&ninja)
        .args([
            "-C",
            final_build_dir.to_str().unwrap(),
            "client:client",
            "client:common",
            "util:util",
            "third_party/mini_chromium/mini_chromium/base:base",
            "handler:crashpad_handler",
        ])
        .current_dir(&crashpad_dir)
        .status()?;

    if !status.success() {
        return Err("ninja build failed".into());
    }

    // Create build completion marker
    let marker_file = final_build_dir.join(".crashpad-ok");
    fs::write(&marker_file, "")?;

    Ok(CrashpadBuildOutput {
        build_out_dir: final_build_dir,
        crashpad_dir,
    })
}

/// Build crashpad-rs-sys wrapper, bindgen, and link setup
/// This reuses phases.rs logic which is already battle-tested
pub fn build_crashpad_sys(
    build_output: &CrashpadBuildOutput,
    target: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    use crate::config::BuildConfig;
    use crate::phases::BuildPhases;

    // Create proper config using from_env() to get all platform-specific settings
    let mut config = BuildConfig::from_env()?;

    // Override paths to point to our depot-built Crashpad
    config.crashpad_dir = build_output.crashpad_dir.clone();

    // Use phases for wrapper compilation, bindgen, and linking
    let phases = BuildPhases::new(config);

    // Skip prepare/configure/build - we already built with depot_tools
    // Just run wrapper, package, bindgen, and emit_link
    phases.wrapper()?;
    phases.package()?;
    phases.bindgen()?;
    phases.emit_link()?;

    // Copy handler to final target directory
    copy_handler_to_target(&build_output.build_out_dir, target)?;

    Ok(())
}

/// Copy crashpad_handler to target directory for distribution
fn copy_handler_to_target(
    build_dir: &Path,
    target: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // iOS doesn't have external handler
    if target.contains("ios") {
        return Ok(());
    }

    let handler_src = if target.contains("windows") {
        build_dir.join("crashpad_handler.exe")
    } else if target.contains("android") {
        build_dir.join("libcrashpad_handler.so")
    } else {
        build_dir.join("crashpad_handler")
    };

    // Skip if handler wasn't built
    if !handler_src.exists() {
        println!(
            "cargo:warning=Handler not found at {}, skipping copy",
            handler_src.display()
        );
        return Ok(());
    }

    // Determine target directory: prefer CARGO_TARGET_DIR else workspace target/
    let host = env::var("HOST").unwrap_or_else(|_| target.to_string());
    let is_cross_compile = host != target;
    let profile = env::var("PROFILE").unwrap_or_else(|_| "debug".to_string());
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?);

    let root = if let Ok(dir) = env::var("CARGO_TARGET_DIR") {
        PathBuf::from(dir)
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

    // Android needs lib prefix and .so extension for APK packaging
    let handler_dest = if target.contains("android") {
        target_dir.join("libcrashpad_handler.so")
    } else if target.contains("windows") {
        target_dir.join("crashpad_handler.exe")
    } else {
        target_dir.join("crashpad_handler")
    };

    println!(
        "cargo:warning=Copying handler from {} to {}",
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

    println!(
        "cargo:rustc-env=CRASHPAD_HANDLER_PATH={}",
        handler_dest.display()
    );
    // Expose handler path to dependents via DEP_<links>_HANDLER
    println!("cargo:handler={}", handler_dest.display());

    Ok(())
}

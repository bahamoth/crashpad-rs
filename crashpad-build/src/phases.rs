/// Build phases management
///
/// This module implements the actual build steps for Crashpad.
/// Each phase is a clearly defined step in the build process.
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

use super::config::BuildConfig;

pub struct BuildPhases {
    config: BuildConfig,
}

impl BuildPhases {
    /// Create a new BuildPhases instance with the given configuration
    pub fn new(config: BuildConfig) -> Self {
        Self { config }
    }

    /// Phase 1: Prepare dependencies (depot_tools, crashpad source)
    pub fn prepare(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Ensure depot_tools is available with correct version
        if !self.config.depot_tools.exists() {
            if self.config.verbose {
                eprintln!("Cloning depot_tools...");
            }

            let status = Command::new("git")
                .args([
                    "clone",
                    "https://chromium.googlesource.com/chromium/tools/depot_tools.git",
                ])
                .arg(&self.config.depot_tools)
                .status()?;

            if !status.success() {
                return Err("Failed to clone depot_tools".into());
            }

            // Checkout specific version
            let depot_tools_commit = self.config.depot_tools_commit();
            if self.config.verbose {
                eprintln!("Checking out depot_tools commit: {depot_tools_commit}");
            }

            let status = Command::new("git")
                .args(["checkout", &depot_tools_commit])
                .current_dir(&self.config.depot_tools)
                .status()?;

            if !status.success() {
                return Err(
                    format!("Failed to checkout depot_tools commit {depot_tools_commit}").into(),
                );
            }
        } else {
            // Check if we have the right version
            let depot_tools_commit = self.config.depot_tools_commit();
            let output = Command::new("git")
                .args(["rev-parse", "HEAD"])
                .current_dir(&self.config.depot_tools)
                .output()?;

            let current = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if current != depot_tools_commit {
                if self.config.verbose {
                    eprintln!(
                        "depot_tools version mismatch. Current: {current}, Expected: {depot_tools_commit}"
                    );
                    eprintln!("Updating depot_tools to correct version...");
                }

                // Fetch and checkout
                Command::new("git")
                    .args(["fetch", "origin"])
                    .current_dir(&self.config.depot_tools)
                    .status()?;

                let status = Command::new("git")
                    .args(["checkout", &depot_tools_commit])
                    .current_dir(&self.config.depot_tools)
                    .status()?;

                if !status.success() {
                    return Err(format!(
                        "Failed to update depot_tools to commit {depot_tools_commit}"
                    )
                    .into());
                }
            }
        }

        // Ensure crashpad is available with correct version
        if !self.config.crashpad_dir.exists() {
            if self.config.verbose {
                eprintln!("Setting up Crashpad...");
            }

            fs::create_dir_all(&self.config.crashpad_checkout)?;

            // Write .gclient configuration
            let gclient_content = r#"solutions = [
  {
    "name": "crashpad",
    "url": "https://chromium.googlesource.com/crashpad/crashpad.git",
    "deps_file": "DEPS",
    "managed": False,
  },
]
"#;
            fs::write(
                self.config.crashpad_checkout.join(".gclient"),
                gclient_content,
            )?;

            // Clone crashpad
            let status = Command::new("git")
                .args([
                    "clone",
                    "https://chromium.googlesource.com/crashpad/crashpad.git",
                ])
                .current_dir(&self.config.crashpad_checkout)
                .status()?;

            if !status.success() {
                return Err("Failed to clone crashpad repository".into());
            }

            // Checkout specific version
            let crashpad_commit = self.config.crashpad_commit();
            if self.config.verbose {
                eprintln!("Checking out crashpad commit: {crashpad_commit}");
            }

            let status = Command::new("git")
                .args(["checkout", &crashpad_commit])
                .current_dir(&self.config.crashpad_dir)
                .status()?;

            if !status.success() {
                return Err(format!("Failed to checkout crashpad commit {crashpad_commit}").into());
            }

            // Run gclient sync
            if self.config.verbose {
                eprintln!("Running gclient sync...");
            }

            let status = Command::new("gclient")
                .args(["sync", "--no-history", "-D"])
                .current_dir(&self.config.crashpad_checkout)
                .env("PATH", self.config.path_with_depot_tools())
                .status()?;

            if !status.success() {
                return Err("Failed to run gclient sync".into());
            }
        } else {
            // Check if we have the right version
            let crashpad_commit = self.config.crashpad_commit();
            let output = Command::new("git")
                .args(["rev-parse", "HEAD"])
                .current_dir(&self.config.crashpad_dir)
                .output()?;

            let current = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if current != crashpad_commit {
                if self.config.verbose {
                    eprintln!(
                        "Crashpad version mismatch. Current: {current}, Expected: {crashpad_commit}"
                    );
                    eprintln!("Updating crashpad to correct version...");
                }

                // Fetch and checkout
                Command::new("git")
                    .args(["fetch", "origin"])
                    .current_dir(&self.config.crashpad_dir)
                    .status()?;

                let status = Command::new("git")
                    .args(["checkout", &crashpad_commit])
                    .current_dir(&self.config.crashpad_dir)
                    .status()?;

                if !status.success() {
                    return Err(
                        format!("Failed to update crashpad to commit {crashpad_commit}").into(),
                    );
                }

                // Re-run gclient sync for the new version
                if self.config.verbose {
                    eprintln!("Running gclient sync for updated version...");
                }

                let status = Command::new("gclient")
                    .args(["sync", "--no-history", "-D"])
                    .current_dir(&self.config.crashpad_checkout)
                    .env("PATH", self.config.path_with_depot_tools())
                    .status()?;

                if !status.success() {
                    return Err("Failed to run gclient sync after version update".into());
                }
            }
        }

        Ok(())
    }

    /// Phase 2: Configure build with GN
    pub fn configure(&self) -> Result<(), Box<dyn std::error::Error>> {
        let build_dir = self.config.build_dir();

        // Create GN args string
        let gn_args = self
            .config
            .gn_args
            .iter()
            .map(|(k, v)| format!("{k} = {v}"))
            .collect::<Vec<_>>()
            .join(" ");

        if self.config.verbose {
            eprintln!("Running GN with args: {gn_args}");
        }

        let status = Command::new("gn")
            .args([
                "gen",
                build_dir.to_str().unwrap(),
                &format!("--args={gn_args}"),
            ])
            .current_dir(&self.config.crashpad_dir)
            .env("PATH", self.config.path_with_depot_tools())
            .status()?;

        if !status.success() {
            return Err("Failed to generate build files with GN".into());
        }

        Ok(())
    }

    /// Phase 3: Build with Ninja
    pub fn build(&self) -> Result<(), Box<dyn std::error::Error>> {
        let build_dir = self.config.build_dir();

        if self.config.verbose {
            eprintln!("Running ninja build...");
        }

        let mut cmd = Command::new("ninja");
        cmd.arg("-C")
            .arg(build_dir.to_str().unwrap())
            .current_dir(&self.config.crashpad_dir)
            .env("PATH", self.config.path_with_depot_tools());

        // iOS requires specific targets
        if self.config.target.contains("ios") {
            for target in [
                "client:client",
                "client:common",
                "handler:common",
                "util:util",
                "util:net",
                "util:mig_output",
                "minidump:format",
                "minidump:minidump",
                "snapshot:context",
                "snapshot:snapshot",
                "third_party/mini_chromium/mini_chromium/base:base",
            ] {
                cmd.arg(target);
            }
        }

        let status = cmd.status()?;

        if !status.success() {
            return Err("Ninja build failed".into());
        }

        // Copy crashpad_handler to target directory for easy access
        self.copy_handler_to_target()?;

        // Create prebuilt bundle with only essential files
        self.create_prebuilt_bundle()?;

        Ok(())
    }

    /// Phase 4: Compile wrapper
    pub fn wrapper(&self) -> Result<(), Box<dyn std::error::Error>> {
        if self.config.verbose {
            eprintln!("Compiling wrapper.cc...");
        }

        let wrapper_cc = self.config.manifest_dir.join("crashpad_wrapper.cc");
        let wrapper_obj = self.config.wrapper_obj_path();

        let mut cmd = Command::new(&self.config.compiler);

        // Add compiler flags
        for flag in &self.config.cxx_flags {
            cmd.arg(flag);
        }

        // Add ios-specific defines
        if self.config.target.contains("ios") {
            cmd.args(["-DTARGET_OS_IOS=1"]);
        }

        // Add include paths
        cmd.args([
            "-I",
            self.config.crashpad_dir.to_str().unwrap(),
            "-I",
            self.config
                .crashpad_dir
                .join("third_party/mini_chromium/mini_chromium")
                .to_str()
                .unwrap(),
        ]);

        // Compile to object file
        cmd.args([
            "-c",
            "-o",
            wrapper_obj.to_str().unwrap(),
            wrapper_cc.to_str().unwrap(),
        ]);

        let status = cmd.status()?;

        if !status.success() {
            return Err("Failed to compile wrapper.cc".into());
        }

        if !wrapper_obj.exists() {
            return Err("Wrapper object file not created".into());
        }

        Ok(())
    }

    /// Phase 5: Create static library
    pub fn package(&self) -> Result<(), Box<dyn std::error::Error>> {
        if self.config.verbose {
            eprintln!("Creating static library...");
        }

        let lib_path = self.config.static_lib_path();
        let wrapper_obj = self.config.wrapper_obj_path();
        let build_dir = self.config.build_dir();

        let status = match self.config.archiver.as_str() {
            "libtool" => {
                let mut cmd = Command::new("libtool");
                cmd.args([
                    "-static",
                    "-o",
                    lib_path.to_str().unwrap(),
                    wrapper_obj.to_str().unwrap(),
                ]);

                // For iOS, include additional libraries
                if self.config.target.contains("ios") {
                    let obj_dir = build_dir.join("obj");
                    let handler_common = obj_dir.join("handler/libcommon.a");
                    let util_net = obj_dir.join("util/libnet.a");

                    if handler_common.exists() {
                        cmd.arg(handler_common.to_str().unwrap());
                    }
                    if util_net.exists() {
                        cmd.arg(util_net.to_str().unwrap());
                    }
                }

                cmd.status()?
            }
            _ => Command::new("ar")
                .args([
                    "rcs",
                    lib_path.to_str().unwrap(),
                    wrapper_obj.to_str().unwrap(),
                ])
                .status()?,
        };

        if !status.success() {
            return Err(format!(
                "Failed to create static library with {}",
                self.config.archiver
            )
            .into());
        }

        if !lib_path.exists() {
            return Err("Static library file not created".into());
        }

        Ok(())
    }

    /// Phase 6: Generate FFI bindings
    pub fn bindgen(&self) -> Result<(), Box<dyn std::error::Error>> {
        if self.config.verbose {
            eprintln!("Generating FFI bindings...");
        }

        let mut builder = bindgen::Builder::default()
            .header(self.config.manifest_dir.join("wrapper.h").to_str().unwrap())
            .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()));

        // Add iOS-specific defines for bindgen
        if self.config.target.contains("ios") {
            builder = builder.clang_arg("-DTARGET_OS_IOS=1");
        }

        // For iOS simulator, specify the correct target
        if self.config.target.contains("ios") && self.config.target.contains("sim") {
            let target_flag = if self.config.target.starts_with("aarch64") {
                "arm64-apple-ios-simulator"
            } else {
                "x86_64-apple-ios-simulator"
            };
            builder = builder.clang_arg("-target").clang_arg(target_flag);
        }

        let bindings = builder
            .generate()
            .map_err(|e| format!("Unable to generate bindings: {e:?}"))?;

        bindings
            .write_to_file(self.config.bindings_path())
            .map_err(|e| format!("Couldn't write bindings: {e}"))?;

        Ok(())
    }

    /// Phase 7: Emit cargo link metadata
    pub fn emit_link(&self) -> Result<(), Box<dyn std::error::Error>> {
        let build_dir = self.config.build_dir();

        // Check if using prebuilt bundle (lib/ exists) or full build (obj/ exists)
        let lib_dir = build_dir.join("lib");
        let obj_dir = build_dir.join("obj");
        let is_prebuilt = lib_dir.exists() && lib_dir.is_dir();

        if is_prebuilt {
            // Prebuilt bundle: search in lib/ directory
            println!("cargo:rustc-link-search=native={}", lib_dir.display());

            // Also add OUT_DIR for wrapper object
            println!(
                "cargo:rustc-link-search=native={}",
                self.config.out_dir.display()
            );

            if self.config.verbose {
                eprintln!("Using prebuilt bundle from: {}", lib_dir.display());
            }
        } else {
            // Full build: search in obj/ subdirectories
            let search_paths = [
                obj_dir.join("client"),
                obj_dir.join("util"),
                obj_dir.join("third_party/mini_chromium/mini_chromium/base"),
                obj_dir.join("minidump"),
                obj_dir.join("snapshot"),
                obj_dir.join("handler"),
                self.config.out_dir.clone(),
            ];

            for path in &search_paths {
                println!("cargo:rustc-link-search=native={}", path.display());
            }

            if self.config.verbose {
                eprintln!("Using full build from: {}", obj_dir.display());
            }
        }

        // Add Android NDK library path for C++ static libs
        if self.config.target.contains("android") {
            // NDK libs are already in the linker path via compiler flags
            // but we might need to add them explicitly for some cases
            if let Ok(ndk) = BuildConfig::find_ndk() {
                let target_triple = if self.config.target.starts_with("x86_64") {
                    "x86_64-linux-android"
                } else if self.config.target.starts_with("aarch64") {
                    "aarch64-linux-android"
                } else if self.config.target.starts_with("armv7") {
                    "arm-linux-androideabi"
                } else {
                    "i686-linux-android"
                };

                let lib_path = ndk
                    .join("toolchains/llvm/prebuilt/linux-x86_64/sysroot/usr/lib")
                    .join(target_triple);

                if lib_path.exists() {
                    println!("cargo:rustc-link-search=native={}", lib_path.display());
                }
            }
        }

        // Link Crashpad libraries
        for lib in &self.config.crashpad_libs {
            println!("cargo:rustc-link-lib=static={lib}");
        }

        // Platform-specific system libraries
        for lib in &self.config.link_libs {
            println!("cargo:rustc-link-lib={lib}");
        }

        // Frameworks (iOS/macOS)
        for framework in &self.config.frameworks {
            println!("cargo:rustc-link-lib=framework={framework}");
        }

        // Special case for macOS
        if self.config.target.contains("darwin") {
            println!("cargo:rustc-link-lib=dylib=bsm");
        }

        // Verify handler exists (for platforms that use external handler)
        if !self.config.target.contains("ios") {
            let handler_path = self.config.handler_path();
            if !handler_path.exists() {
                println!(
                    "cargo:warning=crashpad_handler not found at {}. Crash reports cannot be uploaded automatically.",
                    handler_path.display()
                );
            }
        }

        Ok(())
    }

    /// Copy crashpad_handler to target directory for consistent access
    fn copy_handler_to_target(&self) -> Result<(), Box<dyn std::error::Error>> {
        // iOS doesn't have external handler
        if self.config.target.contains("ios") {
            return Ok(());
        }

        let build_dir = self.config.build_dir();
        let handler_src = build_dir.join("crashpad_handler");

        // Skip if handler wasn't built
        if !handler_src.exists() {
            if self.config.verbose {
                eprintln!(
                    "Handler not found at {}, skipping copy",
                    handler_src.display()
                );
            }
            return Ok(());
        }

        // Calculate target directory
        // Use HOST env var to determine if cross-compiling
        // If HOST != TARGET, then we're cross-compiling
        let host = env::var("HOST").unwrap_or_else(|_| self.config.target.clone());
        let is_cross_compile = host != self.config.target;

        let target_dir = if is_cross_compile {
            // Cross-compilation - include target triple
            self.config
                .manifest_dir
                .parent()
                .ok_or("Failed to get parent directory")?
                .join("target")
                .join(&self.config.target)
                .join(&self.config.profile)
        } else {
            // Native build - use simple path
            self.config
                .manifest_dir
                .parent()
                .ok_or("Failed to get parent directory")?
                .join("target")
                .join(&self.config.profile)
        };

        // Create directory if needed
        fs::create_dir_all(&target_dir)?;

        // Android needs lib prefix and .so extension for APK packaging
        let handler_dest = if self.config.target.contains("android") {
            target_dir.join("libcrashpad_handler.so")
        } else if self.config.target.contains("windows") {
            target_dir.join("crashpad_handler.exe")
        } else {
            target_dir.join("crashpad_handler")
        };

        // Copy the handler
        fs::copy(&handler_src, &handler_dest)?;

        if self.config.verbose {
            eprintln!("Copied crashpad_handler to {}", handler_dest.display());
        }

        // Set executable permissions on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&handler_dest)?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&handler_dest, perms)?;
        }

        // Output the path for downstream use
        println!(
            "cargo:rustc-env=CRASHPAD_HANDLER_PATH={}",
            handler_dest.display()
        );

        Ok(())
    }

    /// Create prebuilt bundle with only essential files for distribution
    /// This extracts the minimal set of files needed for linking from the full build
    pub fn create_prebuilt_bundle(&self) -> Result<(), Box<dyn std::error::Error>> {
        if self.config.verbose {
            eprintln!("Creating prebuilt bundle...");
        }

        let build_dir = self.config.build_dir();
        let bundle_dir = self.config.prebuilt_bundle_dir();

        // Clean and create bundle directory structure
        if bundle_dir.exists() {
            fs::remove_dir_all(&bundle_dir)?;
        }

        let lib_dir = bundle_dir.join("lib");
        let include_dir = bundle_dir.join("include");
        let bin_dir = bundle_dir.join("bin");

        fs::create_dir_all(&lib_dir)?;
        fs::create_dir_all(&include_dir)?;
        fs::create_dir_all(&bin_dir)?;

        // Copy essential libraries only (exclude test libraries)
        let obj_dir = build_dir.join("obj");

        // Define which libraries to include
        let essential_libs = vec![
            ("client", "libclient.a"),
            ("client", "libcommon.a"),
            ("util", "libutil.a"),
            ("minidump", "libformat.a"),
            ("minidump", "libminidump.a"),
            ("snapshot", "libsnapshot.a"),
            ("snapshot", "libcontext.a"),
            ("handler", "libcommon.a"),
            ("third_party/mini_chromium/mini_chromium/base", "libbase.a"),
        ];

        // Platform-specific libraries
        if self.config.target.contains("darwin") || self.config.target.contains("ios") {
            // macOS/iOS need MIG output - check both possible locations
            let mig_lib_paths = vec![
                obj_dir.join("util/libmig_output.a"),
                obj_dir.join("util/mach/libmig_output.a"),
            ];

            let mut found = false;
            for mig_lib in &mig_lib_paths {
                if mig_lib.exists() {
                    let dest = lib_dir.join("libmig_output.a");
                    fs::copy(mig_lib, &dest)?;
                    if self.config.verbose {
                        eprintln!("  Copied: libmig_output.a");
                    }
                    found = true;
                    break;
                }
            }

            if !found && self.config.verbose {
                eprintln!("  Warning: libmig_output.a not found (required for macOS/iOS)");
            }
        }

        if self.config.target.contains("ios") {
            // iOS needs additional network library
            let net_lib = obj_dir.join("util/libnet.a");
            if net_lib.exists() {
                let dest = lib_dir.join("libnet.a");
                fs::copy(&net_lib, &dest)?;
                if self.config.verbose {
                    eprintln!("  Copied: libnet.a");
                }
            }
        }

        // Copy essential libraries
        for (subdir, lib_name) in &essential_libs {
            let src = obj_dir.join(subdir).join(lib_name);
            if src.exists() {
                let dest = lib_dir.join(lib_name);
                fs::copy(&src, &dest)?;
                if self.config.verbose {
                    eprintln!("  Copied: {lib_name}");
                }
            } else if self.config.verbose {
                eprintln!("  Warning: {lib_name} not found");
            }
        }

        // Copy header files needed for wrapper compilation
        let crashpad_dir = self.config.crashpad_dir.clone();
        let header_dirs = vec!["client", "util", "handler", "minidump", "snapshot"];

        for dir in &header_dirs {
            let src_dir = crashpad_dir.join(dir);
            let dest_dir = include_dir.join(dir);

            if src_dir.exists() {
                fs::create_dir_all(&dest_dir)?;
                self.copy_headers(&src_dir, &dest_dir)?;
                if self.config.verbose {
                    eprintln!("  Copied headers: {dir}/");
                }
            }
        }

        // Copy mini_chromium headers
        let mini_chromium_src = crashpad_dir.join("third_party/mini_chromium/mini_chromium");
        if mini_chromium_src.exists() {
            let mini_chromium_dest = include_dir.join("third_party/mini_chromium/mini_chromium");
            fs::create_dir_all(&mini_chromium_dest)?;
            self.copy_directory(&mini_chromium_src, &mini_chromium_dest)?;
            if self.config.verbose {
                eprintln!("  Copied headers: third_party/mini_chromium/");
            }
        }

        // Copy handler binary (except for iOS which uses in-process handler)
        if !self.config.target.contains("ios") {
            let handler_name = if self.config.target.contains("android") {
                "libcrashpad_handler.so"
            } else if self.config.target.contains("windows") {
                "crashpad_handler.exe"
            } else {
                "crashpad_handler"
            };

            let handler_src = build_dir.join("crashpad_handler");
            if handler_src.exists() {
                let handler_dest = bin_dir.join(handler_name);
                fs::copy(&handler_src, &handler_dest)?;

                // Make executable on Unix
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    let mut perms = fs::metadata(&handler_dest)?.permissions();
                    perms.set_mode(0o755);
                    fs::set_permissions(&handler_dest, perms)?;
                }

                if self.config.verbose {
                    eprintln!("  Copied handler: {handler_name}");
                }
            }
        }

        // Create metadata.json
        let metadata = HashMap::from([
            ("crashpad_commit", self.config.crashpad_commit()),
            ("depot_tools_commit", self.config.depot_tools_commit()),
            ("target", self.config.target.clone()),
            ("profile", self.config.profile.clone()),
            ("build_date", chrono::Utc::now().to_rfc3339()),
            ("bundle_version", "1.0".to_string()),
        ]);

        let metadata_path = bundle_dir.join("metadata.json");
        let metadata_json = serde_json::to_string_pretty(&metadata)?;
        fs::write(&metadata_path, metadata_json)?;

        if self.config.verbose {
            eprintln!("Prebuilt bundle created at: {}", bundle_dir.display());

            // Calculate and report bundle size
            let bundle_size = self.calculate_dir_size(&bundle_dir)?;
            let build_size = self.calculate_dir_size(&build_dir)?;
            eprintln!(
                "Bundle size: {:.1} MB (vs full build: {:.1} MB - {:.1}% reduction)",
                bundle_size as f64 / 1_048_576.0,
                build_size as f64 / 1_048_576.0,
                ((build_size - bundle_size) as f64 / build_size as f64) * 100.0
            );
        }

        Ok(())
    }

    /// Helper: Copy header files recursively
    fn copy_headers(&self, src: &Path, dest: &Path) -> Result<(), Box<dyn std::error::Error>> {
        Self::copy_headers_impl(src, dest)
    }

    fn copy_headers_impl(src: &Path, dest: &Path) -> Result<(), Box<dyn std::error::Error>> {
        for entry in fs::read_dir(src)? {
            let entry = entry?;
            let path = entry.path();
            let file_name = entry.file_name();
            let dest_path = dest.join(&file_name);

            if path.is_dir() {
                fs::create_dir_all(&dest_path)?;
                Self::copy_headers_impl(&path, &dest_path)?;
            } else if path.extension().and_then(|s| s.to_str()) == Some("h") {
                fs::copy(&path, &dest_path)?;
            }
        }
        Ok(())
    }

    /// Helper: Copy directory recursively
    fn copy_directory(&self, src: &Path, dest: &Path) -> Result<(), Box<dyn std::error::Error>> {
        Self::copy_directory_impl(src, dest)
    }

    fn copy_directory_impl(src: &Path, dest: &Path) -> Result<(), Box<dyn std::error::Error>> {
        for entry in fs::read_dir(src)? {
            let entry = entry?;
            let path = entry.path();
            let file_name = entry.file_name();
            let dest_path = dest.join(&file_name);

            if path.is_dir() {
                fs::create_dir_all(&dest_path)?;
                Self::copy_directory_impl(&path, &dest_path)?;
            } else {
                fs::copy(&path, &dest_path)?;
            }
        }
        Ok(())
    }

    /// Helper: Calculate directory size recursively
    fn calculate_dir_size(&self, dir: &Path) -> Result<u64, Box<dyn std::error::Error>> {
        Self::calculate_dir_size_impl(dir)
    }

    fn calculate_dir_size_impl(dir: &Path) -> Result<u64, Box<dyn std::error::Error>> {
        let mut size = 0;

        if dir.is_dir() {
            for entry in fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();

                if path.is_dir() {
                    size += Self::calculate_dir_size_impl(&path)?;
                } else {
                    size += entry.metadata()?.len();
                }
            }
        }

        Ok(size)
    }
}

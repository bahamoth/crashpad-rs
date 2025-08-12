use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

use crate::config::BuildConfig;
use crate::tools::BinaryToolManager;

pub struct BuildPhases {
    config: BuildConfig,
    gn_path: Option<PathBuf>,
    ninja_path: Option<PathBuf>,
}

impl BuildPhases {
    /// Create a new BuildPhases instance with the given configuration
    pub fn new(config: BuildConfig) -> Self {
        Self {
            config,
            gn_path: None,
            ninja_path: None,
        }
    }

    /// Phase 1: Prepare dependencies (ensure build tools are available)
    pub fn prepare(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Ensure crashpad directory exists and has required files
        if !self.config.crashpad_dir.exists() {
            return Err("Crashpad directory not found".into());
        }

        // Use BinaryToolManager for GN/Ninja
        if self.config.verbose {
            eprintln!("Setting up binary tools...");
        }

        let tool_manager = BinaryToolManager::new(self.config.verbose)?;

        // Ensure GN and Ninja are available
        let gn_path = tool_manager.ensure_gn()?;
        let ninja_path = tool_manager.ensure_ninja()?;

        // Store paths for later use
        self.gn_path = Some(gn_path.clone());
        self.ninja_path = Some(ninja_path.clone());

        if self.config.verbose {
            eprintln!("GN: {}", gn_path.display());
            eprintln!("Ninja: {}", ninja_path.display());
        }

        // Check if symlinks already exist (created by xtask symlink)
        let test_link = self
            .config
            .crashpad_dir
            .join("third_party/mini_chromium/mini_chromium");

        if test_link.exists() {
            // Symlinks already exist, skip creation
            if self.config.verbose {
                eprintln!("Dependencies already linked, skipping symlink creation");
            }
        } else {
            // Create symlinks/junctions for dependencies
            if self.config.verbose {
                eprintln!("Creating dependency symlinks...");
            }
            self.create_dependency_links()?;
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

        // Get GN path (set in prepare phase)
        let gn_cmd = self
            .gn_path
            .as_ref()
            .ok_or("GN path not set. prepare() phase may have failed")?;

        let mut cmd = Command::new(gn_cmd);
        cmd.args([
            "gen",
            build_dir.to_str().unwrap(),
            &format!("--args={gn_args}"),
        ])
        .current_dir(&self.config.crashpad_dir);

        let status = cmd.status()?;

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

        // Get Ninja path (set in prepare phase)
        let ninja_cmd = self
            .ninja_path
            .as_ref()
            .ok_or("Ninja path not set. prepare() phase may have failed")?;

        let mut cmd = Command::new(ninja_cmd);
        cmd.arg("-C")
            .arg(build_dir.to_str().unwrap())
            .current_dir(&self.config.crashpad_dir);

        // Build only required targets (skip tests)
        if self.config.target.contains("ios") {
            // iOS requires specific targets
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
        } else {
            // For other platforms, build only essential libraries (no tests)
            for target in [
                "client:client",
                "client:common",
                "util:util",
                "minidump:format",
                "minidump:minidump",
                "snapshot:context",
                "snapshot:snapshot",
                "handler:common",
                "third_party/mini_chromium/mini_chromium/base:base",
            ] {
                cmd.arg(target);
            }

            // Add handler executable for non-iOS platforms
            cmd.arg("handler:crashpad_handler");
        }

        let status = cmd.status()?;

        if !status.success() {
            return Err("Ninja build failed".into());
        }

        // Copy crashpad_handler to target directory for easy access
        self.copy_handler_to_target()?;

        Ok(())
    }

    /// Phase 4: Compile wrapper
    pub fn wrapper(&self) -> Result<(), Box<dyn std::error::Error>> {
        if self.config.verbose {
            eprintln!("Compiling wrapper.cc...");
        }

        let wrapper_cc = self.config.manifest_dir.join("crashpad_wrapper.cc");
        let wrapper_obj = self.config.out_dir.join("crashpad_wrapper.o");

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

        let lib_path = self.config.out_dir.join("libcrashpad_wrapper.a");
        let wrapper_obj = self.config.out_dir.join("crashpad_wrapper.o");
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

        // Search in obj/ subdirectories
        let obj_dir = build_dir.join("obj");
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
            eprintln!("Using build from: {}", obj_dir.display());
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
            let handler_path = self.config.build_dir().join("crashpad_handler");
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

    /// Create symlinks/junctions for dependencies
    fn create_dependency_links(&self) -> Result<(), Box<dyn std::error::Error>> {
        let deps = vec![
            ("mini_chromium", "mini_chromium"),
            ("googletest", "googletest"),
            ("zlib", "zlib"),
            ("libfuzzer", "src"),
            ("edo", "edo"),
            ("lss", "lss"), // Linux Syscall Support for Android/Linux
        ];

        for (dep_name, subdir) in deps {
            let target = self.config.manifest_dir.join("third_party").join(dep_name);

            let link = self
                .config
                .crashpad_dir
                .join("third_party")
                .join(dep_name)
                .join(subdir);

            // Skip if link already exists or target doesn't exist
            if link.exists() || !target.exists() {
                continue;
            }

            if self.config.verbose {
                eprintln!(
                    "Creating link for {}: {} -> {}",
                    dep_name,
                    link.display(),
                    target.display()
                );
            }

            // Ensure parent directory exists
            if let Some(parent) = link.parent() {
                fs::create_dir_all(parent)?;
            }

            // Create platform-specific link
            #[cfg(unix)]
            {
                std::os::unix::fs::symlink(&target, &link)?;
            }

            #[cfg(windows)]
            {
                // On Windows, try to create a junction (directory symlink)
                // This doesn't require admin privileges
                std::os::windows::fs::symlink_dir(&target, &link).or_else(|_| {
                    // If junction fails, fall back to copying
                    if self.config.verbose {
                        eprintln!(
                            "Warning: Failed to create junction, copying {} instead",
                            dep_name
                        );
                    }
                    Self::copy_directory_impl(&target, &link)
                })?;
            }
        }

        Ok(())
    }
}

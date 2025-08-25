#![allow(dead_code)]

use std::env;
#[allow(unused_imports)]
use std::fs;
use std::path::PathBuf;
use std::process::Command;

use crate::config::BuildConfig;
use crate::tools::BinaryToolManager;

pub struct BuildPhases {
    config: BuildConfig,
    #[allow(dead_code)]
    gn_path: Option<PathBuf>,
    #[allow(dead_code)]
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

        let tool_manager = BinaryToolManager::new(self.config.verbose)?;
        let gn_path = tool_manager.ensure_gn()?;
        let ninja_path = tool_manager.ensure_ninja()?;
        self.gn_path = Some(gn_path.clone());
        self.ninja_path = Some(ninja_path.clone());

        // Check if symlinks already exist (created by xtask symlink)
        let test_link = self
            .config
            .crashpad_dir
            .join("third_party/mini_chromium/mini_chromium");

        if !test_link.exists() {
            self.create_dependency_links()?;
        }

        Ok(())
    }

    /// Phase 2: Configure build with GN
    pub fn configure(&self) -> Result<(), Box<dyn std::error::Error>> {
        let build_dir = self.config.build_dir();

        // Windows: Create python3.exe symlink if needed
        #[cfg(windows)]
        if self.config.target.contains("msvc") {
            self.setup_python3_alias()?;
        }

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

        let output = cmd.output()?;

        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);

        // Only show GN output if verbose or on error
        if self.config.verbose {
            if !stdout.is_empty() {
                eprintln!("GN output: {stdout}");
            }
            if !stderr.is_empty() {
                eprintln!("GN stderr: {stderr}");
            }
        }

        // Only fail if the command actually failed
        if !output.status.success() {
            return Err(format!(
                "Failed to generate build files with GN. stdout: {stdout}, stderr: {stderr}"
            )
            .into());
        }

        Ok(())
    }

    /// Phase 3: Build with Ninja
    pub fn build(&self) -> Result<(), Box<dyn std::error::Error>> {
        let build_dir = self.config.build_dir();

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

        let output = cmd.output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            // Show build errors
            if !stderr.is_empty() {
                eprintln!("Build error: {stderr}");
            }
            if !stdout.is_empty() && self.config.verbose {
                eprintln!("Build output: {stdout}");
            }
            return Err("Failed to build Crashpad libraries".into());
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

        // Windows: Use cc crate for MSVC compilation
        if self.config.target.contains("windows") {
            let mut build = cc::Build::new();
            build
                .cpp(true)
                .std("c++17")
                .file(&wrapper_cc)
                .include(&self.config.crashpad_dir)
                .include(
                    self.config
                        .crashpad_dir
                        .join("third_party/mini_chromium/mini_chromium"),
                )
                .out_dir(&self.config.out_dir);

            // Windows-specific flags
            build.flag_if_supported("/EHsc");

            // Match the runtime library with what GN is using
            // GN builds with /MDd in debug mode, /MD in release mode
            if self.config.profile == "debug" {
                build.flag("/MDd");
                build.define("_ITERATOR_DEBUG_LEVEL", "2");
            } else {
                build.flag("/MD");
            }

            // Force debug mode for cc crate to match our profile
            if self.config.profile == "debug" {
                build.debug(true);
                build.opt_level(0);
            } else {
                build.debug(false);
                build.opt_level(2);
            }

            // Enable verbose output to debug the issue
            if self.config.verbose {
                build.cargo_metadata(false);
                std::env::set_var("CC_PRINT", "1");
            }

            // Compile the wrapper
            build.try_compile("crashpad_wrapper")?;

            return Ok(());
        }

        // Original code for non-Windows platforms
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

        // Windows: cc crate already created the library
        if self.config.target.contains("windows") {
            let lib_path = self.config.out_dir.join("crashpad_wrapper.lib");
            if !lib_path.exists() {
                return Err(format!("Wrapper library not found at {}", lib_path.display()).into());
            }
            return Ok(());
        }

        // Original code for non-Windows platforms
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

        // Windows: Try to find libclang using cc crate's Visual Studio detection
        #[cfg(windows)]
        {
            if env::var("LIBCLANG_PATH").is_err() {
                // Use cc crate to find Visual Studio
                let build = cc::Build::new();
                let tool = build.try_get_compiler()?;

                // Get the compiler path and derive VS installation from it
                let compiler_path = tool.path();
                if self.config.verbose {
                    eprintln!("Found compiler at: {}", compiler_path.display());
                }

                // Try to find LLVM tools relative to the compiler
                // Compiler is typically at: VS_ROOT\VC\Tools\MSVC\VERSION\bin\HostX64\x64\cl.exe
                // LLVM is typically at: VS_ROOT\VC\Tools\Llvm\x64\bin
                if let Some(vc_root) = compiler_path.ancestors().find(|p| p.ends_with("VC")) {
                    let llvm_path = vc_root.join("Tools").join("Llvm").join("x64").join("bin");
                    if llvm_path.exists() && llvm_path.join("libclang.dll").exists() {
                        env::set_var("LIBCLANG_PATH", &llvm_path);
                        if self.config.verbose {
                            eprintln!("Found libclang at: {}", llvm_path.display());
                        }
                    }
                }
            }
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

        // Windows: Link with debug CRT libraries when in debug mode
        #[cfg(windows)]
        if self.config.target.contains("windows") && self.config.profile == "debug" {
            // Link with MSVCRTD (debug CRT)
            println!("cargo:rustc-link-lib=msvcrtd");
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
            let handler_name = if self.config.target.contains("windows") {
                "crashpad_handler.exe"
            } else {
                "crashpad_handler"
            };
            let handler_path = self.config.build_dir().join(handler_name);
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
        let handler_src = if self.config.target.contains("windows") {
            build_dir.join("crashpad_handler.exe")
        } else {
            build_dir.join("crashpad_handler")
        };

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
                Self::copy_directory(&target, &link)?;
            }
        }

        Ok(())
    }

    /// Copy directory recursively (Windows fallback for symlinks)
    #[cfg(windows)]
    fn copy_directory(
        src: &std::path::Path,
        dst: &std::path::Path,
    ) -> Result<(), Box<dyn std::error::Error>> {
        use std::fs;

        fs::create_dir_all(dst)?;
        for entry in fs::read_dir(src)? {
            let entry = entry?;
            let src_path = entry.path();
            let file_name = entry.file_name();
            let dst_path = dst.join(&file_name);

            if src_path.is_dir() {
                Self::copy_directory(&src_path, &dst_path)?;
            } else {
                fs::copy(&src_path, &dst_path)?;
            }
        }
        Ok(())
    }

    /// Setup python3 alias for Windows
    #[cfg(windows)]
    fn setup_python3_alias(&self) -> Result<(), Box<dyn std::error::Error>> {
        use std::process::Command;

        // First check if python3 already works
        if let Ok(output) = Command::new("python3").arg("--version").output() {
            if output.status.success() {
                if self.config.verbose {
                    eprintln!("python3 already available");
                }
                return Ok(());
            }
        }

        // Find python.exe location
        if let Ok(output) = Command::new("where").arg("python").output() {
            if output.status.success() {
                let paths = String::from_utf8_lossy(&output.stdout);
                for path_str in paths.lines() {
                    let python_path = PathBuf::from(path_str.trim());
                    if python_path.exists() && !path_str.contains("WindowsApps") {
                        // Create python3.exe in the same directory
                        let python_dir = python_path.parent().unwrap();
                        let python3_path = python_dir.join("python3.exe");

                        if !python3_path.exists() {
                            // Copy python.exe to python3.exe
                            fs::copy(&python_path, &python3_path)?;
                            if self.config.verbose {
                                eprintln!("Created python3.exe at: {}", python3_path.display());
                            }
                        }
                        return Ok(());
                    }
                }
            }
        }

        // Python is required for Windows builds (used by GN scripts)
        Err("Python is required for Windows builds".into())
    }
}

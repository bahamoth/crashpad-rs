//! Crashpad build system
//!
//! This module provides a clean, modular build system for Crashpad
//! following Rust idioms and ARCHITECTURE.md principles.

mod build_options;
mod config;
mod platform;

use std::fs;
use std::process::Command;

pub use build_options::BuildOptions;
pub use config::{BuildConfig, ConfigError};
pub use platform::Platform;

#[derive(Debug)]
pub struct BuildError(String);

impl std::fmt::Display for BuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Build error: {}", self.0)
    }
}

impl std::error::Error for BuildError {}

impl From<ConfigError> for BuildError {
    fn from(err: ConfigError) -> Self {
        BuildError(err.to_string())
    }
}

impl From<std::io::Error> for BuildError {
    fn from(err: std::io::Error) -> Self {
        BuildError(err.to_string())
    }
}

/// Main builder for Crashpad
pub struct CrashpadBuilder {
    config: BuildConfig,
    options: BuildOptions,
}

impl CrashpadBuilder {
    /// Create a new builder instance
    pub fn new() -> Result<Self, BuildError> {
        Ok(Self {
            config: BuildConfig::from_env()?,
            options: BuildOptions::from_env(),
        })
    }

    /// Run the complete build process
    pub fn build(self) -> Result<(), BuildError> {
        println!("cargo:rerun-if-changed=build.rs");
        println!("cargo:rerun-if-changed=wrapper.h");
        println!("cargo:rerun-if-changed=crashpad_wrapper.cc");

        self.ensure_depot_tools()?;
        self.ensure_crashpad()?;
        self.configure_and_build()?;
        self.compile_wrapper()?;
        self.create_static_library()?;
        self.generate_bindings()?;
        self.setup_linking()?;

        Ok(())
    }

    /// Ensure depot_tools is available
    fn ensure_depot_tools(&self) -> Result<(), BuildError> {
        if !self.config.depot_tools_path.exists() {
            if self.config.verbose() {
                eprintln!("Cloning depot_tools...");
            }

            let status = Command::new("git")
                .args([
                    "clone",
                    "https://chromium.googlesource.com/chromium/tools/depot_tools.git",
                ])
                .arg(&self.config.depot_tools_path)
                .status()
                .map_err(|e| {
                    BuildError(format!(
                        "Failed to execute git command for depot_tools: {e}\n\
                     Make sure git is installed and in your PATH."
                    ))
                })?;

            if !status.success() {
                return Err(BuildError(format!(
                    "Failed to clone depot_tools to {}. \n\
                     Please check your internet connection and ensure you have git installed.",
                    self.config.depot_tools_path.display()
                )));
            }
        }
        Ok(())
    }

    /// Ensure Crashpad is available and synced
    fn ensure_crashpad(&self) -> Result<(), BuildError> {
        if !self.config.crashpad_dir.exists() {
            if self.config.verbose() {
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
                .status()
                .map_err(|e| {
                    BuildError(format!(
                        "Failed to execute git command for crashpad: {}\n\
                     Working directory: {}",
                        e,
                        self.config.crashpad_checkout.display()
                    ))
                })?;

            if !status.success() {
                return Err(BuildError(
                    "Failed to clone crashpad repository. \n\
                     Please check your internet connection and try again."
                        .to_string(),
                ));
            }

            // Run gclient sync
            if self.config.verbose() {
                eprintln!("Running gclient sync...");
            }
            let status = Command::new("gclient")
                .args(["sync", "--no-history", "-D"])
                .current_dir(&self.config.crashpad_checkout)
                .env("PATH", self.config.path_with_depot_tools())
                .status()
                .map_err(|e| {
                    BuildError(format!(
                        "Failed to execute gclient sync: {e}\n\
                     Make sure Python is installed and depot_tools is properly set up."
                    ))
                })?;

            if !status.success() {
                return Err(BuildError(format!(
                    "Failed to run gclient sync. \n\
                     Make sure Python is installed and depot_tools is in PATH: {}\n\
                     You may need to run: export PATH={}:$PATH",
                    self.config.depot_tools_path.display(),
                    self.config.depot_tools_path.display()
                )));
            }
        }
        Ok(())
    }

    /// Configure and build Crashpad
    fn configure_and_build(&self) -> Result<(), BuildError> {
        let build_dir = self.config.build_dir();
        let target = std::env::var("TARGET").unwrap_or_default();

        // Generate GN args from BuildOptions
        let gn_args = self
            .options
            .gn_args(&target)
            .into_iter()
            .map(|(k, v)| format!("{k} = {v}"))
            .collect::<Vec<_>>()
            .join(" ");

        // Run gn gen
        if self.config.verbose() {
            eprintln!(
                "Running gn gen for {}...",
                self.config.platform.build_name()
            );
            eprintln!("GN args: {gn_args}");
        }

        let status = Command::new("gn")
            .args([
                "gen",
                build_dir.to_str().unwrap(),
                &format!("--args={gn_args}"),
            ])
            .current_dir(&self.config.crashpad_dir)
            .env("PATH", self.config.path_with_depot_tools())
            .status()
            .map_err(|e| {
                BuildError(format!(
                    "Failed to execute gn command: {}\n\
                 Working directory: {}",
                    e,
                    self.config.crashpad_dir.display()
                ))
            })?;

        if !status.success() {
            return Err(BuildError(format!(
                "Failed to generate build files with gn for {}.\n\
                 This usually means depot_tools is not properly set up.\n\
                 Try running: export PATH={}:$PATH",
                self.config.platform.build_name(),
                self.config.depot_tools_path.display()
            )));
        }

        // Run ninja
        if self.config.verbose() {
            eprintln!("Running ninja...");
        }
        let mut ninja_cmd = Command::new("ninja");
        ninja_cmd
            .arg("-C")
            .arg(build_dir.to_str().unwrap())
            .current_dir(&self.config.crashpad_dir)
            .env("PATH", self.config.path_with_depot_tools());

        // Add specific targets for iOS
        if target.contains("ios") {
            // iOS requires specific targets
            for ios_target in [
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
                ninja_cmd.arg(ios_target);
            }
        }

        let status = ninja_cmd.status().map_err(|e| {
            BuildError(format!(
                "Failed to execute ninja command: {}\n\
                 Build directory: {}",
                e,
                build_dir.display()
            ))
        })?;

        if !status.success() {
            return Err(BuildError(format!(
                "Ninja build failed for {}.\n\
                 Check the build output above for specific errors.\n\
                 Common issues:\n\
                 - Missing dependencies (run 'gclient sync' again)\n\
                 - Compiler errors (check your C++ compiler installation)",
                self.config.platform.build_name()
            )));
        }

        Ok(())
    }

    /// Compile the wrapper C++ code
    fn compile_wrapper(&self) -> Result<(), BuildError> {
        if self.config.verbose() {
            eprintln!("Compiling wrapper.cc...");
        }

        let wrapper_obj = self.config.wrapper_obj_path();
        let wrapper_cc = self.config.manifest_dir.join("crashpad_wrapper.cc");
        let target = std::env::var("TARGET").unwrap_or_default();

        // Get compiler from BuildOptions
        let compiler = self.options.get_compiler(&target);
        println!("cargo:warning=Using compiler: {}", compiler.display());

        let mut cc_cmd = Command::new(&compiler);

        // Add compile flags from BuildOptions
        cc_cmd.args(self.options.compiler_flags());

        // Add include paths
        cc_cmd.args([
            "-I",
            self.config.crashpad_dir.to_str().unwrap(),
            "-I",
            self.config
                .crashpad_dir
                .join("third_party/mini_chromium/mini_chromium")
                .to_str()
                .unwrap(),
        ]);

        // Add clang target for iOS
        if target.contains("ios") {
            if target.contains("sim") {
                if target.starts_with("aarch64") {
                    cc_cmd.arg("-target").arg("arm64-apple-ios14.0-simulator");
                } else if target.starts_with("x86_64") {
                    cc_cmd.arg("-target").arg("x86_64-apple-ios14.0-simulator");
                }
            } else if target.starts_with("aarch64") {
                cc_cmd.arg("-target").arg("arm64-apple-ios14.0");
            }
        }

        // Output and input files
        cc_cmd.args([
            "-o",
            wrapper_obj.to_str().unwrap(),
            wrapper_cc.to_str().unwrap(),
        ]);

        let status = cc_cmd.status().map_err(|e| {
            BuildError(format!(
                "Failed to execute C++ compiler: {}\n\
                 Compiler: c++\n\
                 Input file: {}",
                e,
                self.config
                    .manifest_dir
                    .join("crashpad_wrapper.cc")
                    .display()
            ))
        })?;

        if !status.success() {
            return Err(BuildError(format!(
                "Failed to compile wrapper.cc.\n\
                 Make sure you have a C++ compiler installed:\n\
                 - Linux: sudo apt install build-essential\n\
                 - macOS: xcode-select --install\n\
                 Platform: {}",
                self.config.platform.build_name()
            )));
        }

        if !wrapper_obj.exists() {
            return Err(BuildError(format!(
                "wrapper.cc compilation succeeded but object file not found at: {}\n\
                 This might be a permission issue or disk space problem.",
                wrapper_obj.display()
            )));
        }

        Ok(())
    }

    /// Create static library from wrapper object
    fn create_static_library(&self) -> Result<(), BuildError> {
        let lib_path = self.config.static_lib_path();
        let wrapper_obj = self.config.wrapper_obj_path();
        let obj_dir = self.config.obj_dir();
        let target = std::env::var("TARGET").unwrap_or_default();

        // Get archiver from BuildOptions
        let archiver = self.options.get_archiver(&target);

        let status = match archiver.as_str() {
            "libtool" => {
                let mut cmd = Command::new("libtool");
                cmd.args([
                    "-static",
                    "-o",
                    lib_path.to_str().unwrap(),
                    wrapper_obj.to_str().unwrap(),
                ]);

                // For iOS, include additional libraries to avoid linking issues
                if target.contains("ios") {
                    let handler_common = obj_dir.join("handler/libcommon.a");
                    let util_net = obj_dir.join("util/libnet.a");

                    if handler_common.exists() {
                        cmd.arg(handler_common.to_str().unwrap());
                    }
                    if util_net.exists() {
                        cmd.arg(util_net.to_str().unwrap());
                    }
                }

                cmd.status()
                    .map_err(|e| BuildError(format!("Failed to create static library: {e}")))?
            }
            "ar" => Command::new("ar")
                .args([
                    "rcs",
                    lib_path.to_str().unwrap(),
                    wrapper_obj.to_str().unwrap(),
                ])
                .status()
                .map_err(|e| BuildError(format!("Failed to create static library: {e}")))?,
            tool => {
                return Err(BuildError(format!("Unknown archiver tool: {tool}")));
            }
        };

        if !status.success() {
            return Err(BuildError(format!(
                "Failed to create static library with {archiver}.\n\
                 Make sure you have the required tools installed:\n\
                 - macOS/iOS: Xcode Command Line Tools\n\
                 - Linux: binutils package"
            )));
        }

        if !lib_path.exists() {
            return Err(BuildError(format!(
                "Static library creation succeeded but file not found at: {}\n\
                 This might be a permission issue.",
                lib_path.display()
            )));
        }

        Ok(())
    }

    /// Generate Rust bindings
    fn generate_bindings(&self) -> Result<(), BuildError> {
        let mut builder = bindgen::Builder::default()
            .header("wrapper.h")
            .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()));

        // For iOS simulator, specify the correct target
        if let Platform::Ios {
            simulator: true,
            arch,
            ..
        } = &self.config.platform
        {
            builder = builder.clang_arg("-target").clang_arg(match arch {
                platform::Arch::Arm64 => "arm64-apple-ios-simulator",
                platform::Arch::X64 => "x86_64-apple-ios-simulator",
                _ => {
                    return Err(BuildError(format!(
                        "Unsupported iOS simulator architecture: {arch:?}\n\
                     Supported architectures: arm64, x86_64"
                    )))
                }
            });
        }

        let bindings = builder
            .generate()
            .map_err(|e| BuildError(format!("Unable to generate bindings: {e:?}")))?;

        bindings
            .write_to_file(self.config.bindings_path())
            .map_err(|e| BuildError(format!("Couldn't write bindings: {e}")))?;

        Ok(())
    }

    /// Setup linking configuration
    fn setup_linking(&self) -> Result<(), BuildError> {
        let obj_dir = self.config.obj_dir();

        // Library search paths
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

        let target = std::env::var("TARGET").unwrap_or_default();

        // Add Android NDK library path for C++ static libs
        if target.contains("android") {
            if let Some(ndk) = &self.options.ndk_path {
                let target_triple = if target.starts_with("x86_64") {
                    "x86_64-linux-android"
                } else if target.starts_with("aarch64") {
                    "aarch64-linux-android"
                } else if target.starts_with("armv7") {
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

        // Link libraries with proper prefix based on LinkType
        let link_prefix = self.options.link_prefix();

        // Crashpad libraries
        for lib in [
            "crashpad_wrapper",
            "client",
            "common",
            "util",
            "format",
            "minidump",
            "snapshot",
            "context",
            "base",
        ] {
            println!("cargo:rustc-link-lib={link_prefix}{lib}");
        }

        // Platform-specific system libraries
        if target.contains("android") {
            println!("cargo:rustc-link-lib=c++_static");
            println!("cargo:rustc-link-lib=c++abi");
            println!("cargo:rustc-link-lib=log");
            println!("cargo:rustc-link-lib=dl");
        } else if target.contains("linux") {
            println!("cargo:rustc-link-lib=stdc++");
            println!("cargo:rustc-link-lib=pthread");
        } else if target.contains("apple-ios") {
            println!("cargo:rustc-link-lib=c++");
            println!("cargo:rustc-link-lib=z");
            println!("cargo:rustc-link-lib=framework=Foundation");
            println!("cargo:rustc-link-lib=framework=Security");
            println!("cargo:rustc-link-lib=framework=CoreFoundation");
            println!("cargo:rustc-link-lib=framework=UIKit");
        } else if target.contains("apple") {
            println!("cargo:rustc-link-lib=c++");
            println!("cargo:rustc-link-lib=framework=Foundation");
            println!("cargo:rustc-link-lib=framework=Security");
            println!("cargo:rustc-link-lib=framework=CoreFoundation");
            println!("cargo:rustc-link-lib=framework=IOKit");
            println!("cargo:rustc-link-lib=dylib=bsm");
        }

        // Verify handler exists (for platforms that use external handler)
        if !target.contains("ios") {
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
}

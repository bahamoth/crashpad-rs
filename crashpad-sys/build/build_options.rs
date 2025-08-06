//! Build options management
//!
//! This module handles build configuration options from environment variables
//! and provides them in a structured format for the build process.

use std::env;
use std::path::PathBuf;

/// Core build options
#[derive(Debug, Clone)]
pub struct BuildOptions {
    /// Build type: debug/release
    pub build_type: BuildType,

    /// Link type: static/shared
    pub link_type: LinkType,

    /// Compiler path (None = system default)
    pub compiler: Option<PathBuf>,

    /// Archiver path (None = system default)
    pub archiver: Option<PathBuf>,

    /// NDK path (Android only)
    pub ndk_path: Option<PathBuf>,

    /// Additional compile flags
    pub extra_flags: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BuildType {
    Debug,
    Release,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LinkType {
    Static,
    Shared,
}

impl BuildOptions {
    /// Collect options from environment variables and target
    pub fn from_env() -> Self {
        let target = env::var("TARGET").unwrap_or_default();
        let profile = env::var("PROFILE").unwrap_or_else(|_| "debug".to_string());

        // Set defaults
        let mut options = Self {
            build_type: if profile == "release" {
                BuildType::Release
            } else {
                BuildType::Debug
            },
            link_type: LinkType::Static, // default
            compiler: None,
            archiver: None,
            ndk_path: None,
            extra_flags: vec![],
        };

        // Mobile platforms force static linking
        let is_mobile = target.contains("android") || target.contains("ios");

        // Link type (only changeable for non-mobile)
        if !is_mobile {
            if let Ok(link) = env::var("CRASHPAD_LINK_TYPE") {
                options.link_type = match link.as_str() {
                    "shared" | "dynamic" => LinkType::Shared,
                    _ => LinkType::Static,
                };
            }
        }

        // Compiler override
        if let Ok(cxx) = env::var("CXX") {
            options.compiler = Some(PathBuf::from(cxx));
        }

        // Archiver override
        if let Ok(ar) = env::var("AR") {
            options.archiver = Some(PathBuf::from(ar));
        }

        // Android NDK path
        if target.contains("android") {
            options.ndk_path = env::var("ANDROID_NDK_HOME")
                .or_else(|_| env::var("ANDROID_NDK_ROOT"))
                .or_else(|_| env::var("NDK_HOME"))
                .ok()
                .map(PathBuf::from);
        }

        // Extra flags
        if let Ok(flags) = env::var("CRASHPAD_EXTRA_FLAGS") {
            options.extra_flags = flags.split_whitespace().map(String::from).collect();
        }

        options
    }

    /// Generate GN args for Crashpad build
    pub fn gn_args(&self, target: &str) -> Vec<(String, String)> {
        let mut args = vec![];

        // Build type
        args.push((
            "is_debug".to_string(),
            matches!(self.build_type, BuildType::Debug).to_string(),
        ));

        // Link type
        args.push((
            "is_component_build".to_string(),
            matches!(self.link_type, LinkType::Shared).to_string(),
        ));

        // Platform-specific settings
        if target.contains("android") {
            args.push(("target_os".to_string(), "\"android\"".to_string()));
            if let Some(ndk) = &self.ndk_path {
                args.push((
                    "android_ndk_root".to_string(),
                    format!("\"{}\"", ndk.display()),
                ));
            }

            // API level
            args.push(("android_api_level".to_string(), "21".to_string()));
        } else if target.contains("ios") {
            args.push(("target_os".to_string(), "\"ios\"".to_string()));

            if target.contains("sim") {
                args.push((
                    "target_environment".to_string(),
                    "\"simulator\"".to_string(),
                ));
            }
        }

        // CPU architecture
        let cpu = if target.starts_with("x86_64") {
            "x64"
        } else if target.starts_with("aarch64") {
            "arm64"
        } else if target.starts_with("armv7") {
            "arm"
        } else if target.starts_with("i686") {
            "x86"
        } else {
            "x64" // default
        };
        args.push(("target_cpu".to_string(), format!("\"{cpu}\"")));

        args
    }

    /// Get compiler path for wrapper compilation
    pub fn get_compiler(&self, target: &str) -> PathBuf {
        // Use explicit override if provided
        if let Some(compiler) = &self.compiler {
            return compiler.clone();
        }

        // Android uses NDK compiler
        if target.contains("android") {
            if let Some(ndk) = &self.ndk_path {
                let compiler_name = if target.starts_with("x86_64") {
                    "x86_64-linux-android21-clang++"
                } else if target.starts_with("aarch64") {
                    "aarch64-linux-android21-clang++"
                } else if target.starts_with("armv7") {
                    "armv7a-linux-androideabi21-clang++"
                } else {
                    "i686-linux-android21-clang++"
                };

                let path = ndk
                    .join("toolchains/llvm/prebuilt/linux-x86_64/bin")
                    .join(compiler_name);

                if path.exists() {
                    return path;
                } else {
                    println!(
                        "cargo:warning=NDK compiler not found at: {}",
                        path.display()
                    );
                }
            }
        }

        // Default
        PathBuf::from("c++")
    }

    /// Get archiver tool
    pub fn get_archiver(&self, target: &str) -> String {
        if let Some(archiver) = &self.archiver {
            archiver.to_string_lossy().into_owned()
        } else if target.contains("apple") {
            "libtool".to_string()
        } else {
            "ar".to_string()
        }
    }

    /// Get compiler flags for wrapper
    pub fn compiler_flags(&self) -> Vec<String> {
        let mut flags = vec!["-c".to_string(), "-std=c++17".to_string()];

        // Build type
        match self.build_type {
            BuildType::Debug => flags.push("-g".to_string()),
            BuildType::Release => flags.push("-O2".to_string()),
        }

        // Link type
        if matches!(self.link_type, LinkType::Shared) {
            flags.push("-fPIC".to_string());
        }

        // Extra flags
        flags.extend(self.extra_flags.clone());

        flags
    }

    /// Get library link prefix for Rust
    pub fn link_prefix(&self) -> &'static str {
        match self.link_type {
            LinkType::Static => "static=",
            LinkType::Shared => "dylib=",
        }
    }
}

/// Platform configuration management
///
/// This module centralizes all platform-specific build settings.
/// It detects the target platform and configures all necessary build parameters
/// in one place, ensuring consistency between GN args and compiler flags.
use std::collections::HashMap;
use std::env;
use std::path::PathBuf;
use std::process::Command;

/// Native dependencies versions
/// These are pinned to ensure reproducible builds across environments.
/// Can be overridden with environment variables CRASHPAD_COMMIT and DEPOT_TOOLS_COMMIT.
pub const CRASHPAD_COMMIT: &str = "811b04296520206655bf9bfde5e800181a9282f6";
pub const DEPOT_TOOLS_COMMIT: &str = "322a071997b51e483fac86d4f61a98934950923e";

#[derive(Debug, Clone)]
pub struct BuildConfig {
    // Basic information
    pub target: String,
    pub profile: String,
    pub out_dir: PathBuf,
    pub manifest_dir: PathBuf,

    // Paths
    pub depot_tools: PathBuf,
    pub crashpad_checkout: PathBuf,
    pub crashpad_dir: PathBuf,

    // Platform-specific unified settings
    pub compiler: PathBuf,
    pub archiver: String,
    pub cxx_flags: Vec<String>,
    pub gn_args: HashMap<String, String>,
    pub link_libs: Vec<String>,
    pub frameworks: Vec<String>, // iOS/macOS only

    // Build options
    pub verbose: bool,
}

impl BuildConfig {
    /// Create configuration from environment variables
    pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        let target = env::var("TARGET")?;
        let profile = env::var("PROFILE").unwrap_or_else(|_| "debug".to_string());
        let out_dir = PathBuf::from(env::var("OUT_DIR")?);
        let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?);

        // Set up common paths
        let third_party = manifest_dir
            .parent()
            .ok_or("Failed to get parent directory")?
            .join("third_party");
        let depot_tools = third_party.join("depot_tools");
        let crashpad_checkout = third_party.join("crashpad_checkout");
        let crashpad_dir = crashpad_checkout.join("crashpad");

        let mut config = Self {
            target: target.clone(),
            profile: profile.clone(),
            out_dir,
            manifest_dir,
            depot_tools,
            crashpad_checkout,
            crashpad_dir,
            compiler: PathBuf::from("c++"),
            archiver: "ar".to_string(),
            cxx_flags: vec!["-std=c++17".to_string()],
            gn_args: HashMap::new(),
            link_libs: vec!["stdc++".to_string(), "pthread".to_string()],
            frameworks: Vec::new(),
            verbose: env::var("CRASHPAD_VERBOSE").is_ok(),
        };

        // Common GN args
        config.gn_args.insert(
            "is_debug".to_string(),
            if profile == "release" {
                "false"
            } else {
                "true"
            }
            .to_string(),
        );

        // Platform-specific configuration
        if target.contains("android") {
            config.setup_android(&target)?;
        } else if target.contains("ios") {
            config.setup_ios(&target);
        } else if target.contains("darwin") {
            config.setup_macos(&target);
        } else if target.contains("windows") {
            config.setup_windows(&target)?;
        } else if target.contains("linux") {
            config.setup_linux(&target);
        } else {
            return Err(format!("Unsupported target: {target}").into());
        }

        Ok(config)
    }

    /// Configure for Android
    fn setup_android(&mut self, target: &str) -> Result<(), Box<dyn std::error::Error>> {
        // Find NDK dynamically
        let ndk = Self::find_ndk()?;

        // Determine architecture and triple
        let (arch, triple, api) = if target.starts_with("aarch64") {
            ("arm64", "aarch64-linux-android", 21)
        } else if target.starts_with("x86_64") {
            ("x64", "x86_64-linux-android", 21)
        } else if target.starts_with("armv7") {
            ("arm", "arm-linux-androideabi", 21)
        } else if target.starts_with("i686") {
            ("x86", "i686-linux-android", 21)
        } else {
            return Err(format!("Unsupported Android architecture: {target}").into());
        };

        // Compiler path (dynamic, not hardcoded)
        self.compiler = ndk
            .join("toolchains/llvm/prebuilt/linux-x86_64/bin")
            .join(format!("{triple}{api}-clang++"));

        // GN args
        self.gn_args
            .insert("target_os".to_string(), "\"android\"".to_string());
        self.gn_args
            .insert("target_cpu".to_string(), format!("\"{arch}\""));
        self.gn_args.insert(
            "android_ndk_root".to_string(),
            format!("\"{}\"", ndk.display()),
        );
        self.gn_args
            .insert("android_api_level".to_string(), api.to_string());
        self.gn_args
            .insert("is_component_build".to_string(), "false".to_string());

        // Compiler flags (matching GN settings)
        self.cxx_flags = vec![
            format!("--target={}{}", triple, api),
            "-std=c++17".to_string(),
            "-fno-exceptions".to_string(),
            "-fno-rtti".to_string(),
            "-DANDROID".to_string(),
        ];

        // Link libraries
        self.link_libs = vec![
            "c++_static".to_string(),
            "c++abi".to_string(),
            "log".to_string(),
            "dl".to_string(),
        ];

        Ok(())
    }

    /// Configure for iOS
    fn setup_ios(&mut self, target: &str) {
        let (arch, ios_target) = if target.contains("sim") {
            if target.starts_with("aarch64") {
                ("arm64", "arm64-apple-ios14.0-simulator")
            } else {
                ("x64", "x86_64-apple-ios14.0-simulator")
            }
        } else if target.starts_with("aarch64") {
            ("arm64", "arm64-apple-ios14.0")
        } else {
            ("x64", "x86_64-apple-ios14.0")
        };

        // GN args
        self.gn_args
            .insert("target_os".to_string(), "\"ios\"".to_string());
        self.gn_args
            .insert("target_cpu".to_string(), format!("\"{arch}\""));
        if target.contains("sim") {
            self.gn_args.insert(
                "target_environment".to_string(),
                "\"simulator\"".to_string(),
            );
        }

        // Compiler settings
        self.compiler = PathBuf::from("clang++");
        self.cxx_flags = vec![
            "-std=c++17".to_string(),
            format!("--target={}", ios_target),
            "-fno-exceptions".to_string(),
            "-fno-rtti".to_string(),
        ];

        // Add iOS SDK path
        if let Ok(output) = Command::new("xcrun")
            .args([
                "--sdk",
                if target.contains("sim") {
                    "iphonesimulator"
                } else {
                    "iphoneos"
                },
                "--show-sdk-path",
            ])
            .output()
        {
            if output.status.success() {
                if let Ok(sdk_path) = String::from_utf8(output.stdout) {
                    self.cxx_flags.push("-isysroot".to_string());
                    self.cxx_flags.push(sdk_path.trim().to_string());
                }
            }
        }

        // iOS uses libtool
        self.archiver = "libtool".to_string();

        // Link settings
        self.link_libs = vec!["c++".to_string(), "z".to_string()];
        self.frameworks = vec![
            "Foundation".to_string(),
            "Security".to_string(),
            "CoreFoundation".to_string(),
            "UIKit".to_string(),
        ];
    }

    /// Configure for macOS
    fn setup_macos(&mut self, target: &str) {
        let arch = if target.starts_with("aarch64") {
            "arm64"
        } else {
            "x64"
        };

        self.gn_args
            .insert("target_os".to_string(), "\"mac\"".to_string());
        self.gn_args
            .insert("target_cpu".to_string(), format!("\"{arch}\""));

        self.compiler = PathBuf::from("c++");
        self.archiver = "libtool".to_string();

        self.link_libs = vec!["c++".to_string()];
        self.frameworks = vec![
            "Foundation".to_string(),
            "Security".to_string(),
            "CoreFoundation".to_string(),
            "IOKit".to_string(),
        ];
    }

    /// Configure for Windows
    fn setup_windows(&mut self, target: &str) -> Result<(), Box<dyn std::error::Error>> {
        let arch = if target.starts_with("x86_64") {
            "x64"
        } else if target.starts_with("i686") {
            "x86"
        } else {
            return Err(format!("Unsupported Windows architecture: {target}").into());
        };

        self.gn_args
            .insert("target_os".to_string(), "\"win\"".to_string());
        self.gn_args
            .insert("target_cpu".to_string(), format!("\"{arch}\""));

        // For cross-compilation from Linux
        if !target.contains("msvc") {
            // MinGW settings
            self.compiler = PathBuf::from("x86_64-w64-mingw32-g++");
            self.archiver = "x86_64-w64-mingw32-ar".to_string();
        }

        Ok(())
    }

    /// Configure for Linux
    fn setup_linux(&mut self, target: &str) {
        let arch = if target.starts_with("x86_64") {
            "x64"
        } else if target.starts_with("aarch64") {
            "arm64"
        } else if target.starts_with("armv7") {
            "arm"
        } else {
            "x86"
        };

        self.gn_args
            .insert("target_os".to_string(), "\"linux\"".to_string());
        self.gn_args
            .insert("target_cpu".to_string(), format!("\"{arch}\""));

        // Add PIC flag for Linux
        self.cxx_flags.push("-fPIC".to_string());
    }

    /// Find Android NDK path dynamically
    pub fn find_ndk() -> Result<PathBuf, Box<dyn std::error::Error>> {
        // Priority: cargo-ndk > environment variables > standard paths

        // 1. Check cargo-ndk environment variable
        if let Ok(path) = env::var("CARGO_NDK_ANDROID_NDK_HOME") {
            let path = PathBuf::from(path);
            if path.exists() {
                return Ok(path);
            }
        }

        // 2. Check standard environment variables
        for var in ["ANDROID_NDK_HOME", "ANDROID_NDK_ROOT", "NDK_HOME"] {
            if let Ok(path) = env::var(var) {
                let path = PathBuf::from(path);
                if path.exists() {
                    return Ok(path);
                }
            }
        }

        // 3. Check standard installation paths
        if let Ok(home) = env::var("HOME") {
            let candidates = vec![
                PathBuf::from(&home).join("Android/Sdk/ndk-bundle"),
                PathBuf::from(&home).join("Library/Android/sdk/ndk-bundle"),
                PathBuf::from(&home).join("android-ndk"),
            ];

            for candidate in candidates {
                if candidate.exists() {
                    return Ok(candidate);
                }
            }
        }

        // 4. Check system paths
        let system_paths = vec![
            PathBuf::from("/opt/android-ndk"),
            PathBuf::from("/usr/local/android-ndk"),
        ];

        for path in system_paths {
            if path.exists() {
                return Ok(path);
            }
        }

        Err("Android NDK not found. Please set ANDROID_NDK_HOME or install cargo-ndk".into())
    }

    /// Get build directory for current platform
    pub fn build_dir(&self) -> PathBuf {
        self.out_dir.join("crashpad_build")
    }

    /// Get wrapper object file path
    pub fn wrapper_obj_path(&self) -> PathBuf {
        self.out_dir.join("crashpad_wrapper.o")
    }

    /// Get static library path
    pub fn static_lib_path(&self) -> PathBuf {
        self.out_dir.join("libcrashpad_wrapper.a")
    }

    /// Get bindings output path
    pub fn bindings_path(&self) -> PathBuf {
        self.out_dir.join("bindings.rs")
    }

    /// Get handler binary path
    pub fn handler_path(&self) -> PathBuf {
        self.build_dir().join("crashpad_handler")
    }

    /// Get PATH with depot_tools
    pub fn path_with_depot_tools(&self) -> String {
        format!(
            "{}:{}",
            self.depot_tools.display(),
            env::var("PATH").unwrap_or_default()
        )
    }

    /// Get crashpad commit to use (with env override)
    pub fn crashpad_commit(&self) -> String {
        env::var("CRASHPAD_COMMIT").unwrap_or_else(|_| CRASHPAD_COMMIT.to_string())
    }

    /// Get depot_tools commit to use (with env override)
    pub fn depot_tools_commit(&self) -> String {
        env::var("DEPOT_TOOLS_COMMIT").unwrap_or_else(|_| DEPOT_TOOLS_COMMIT.to_string())
    }
}

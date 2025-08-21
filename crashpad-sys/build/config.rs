/// Platform configuration management
///
/// This module centralizes all platform-specific build settings.
/// It detects the target platform and configures all necessary build parameters
/// in one place, ensuring consistency between GN args and compiler flags.
use std::collections::HashMap;
use std::env;
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug, Clone)]
pub struct BuildConfig {
    // Basic information
    pub target: String,
    pub profile: String,
    pub out_dir: PathBuf,
    pub manifest_dir: PathBuf,

    // Paths
    pub crashpad_dir: PathBuf,

    // Compiler settings (for wrapper compilation with cc crate)
    pub compiler: PathBuf,
    pub archiver: String,
    pub cxx_flags: Vec<String>,
    
    // GN build settings (only for vendored build, not depot_tools)
    pub gn_args: HashMap<String, String>,
    
    // Linking settings (always needed)
    pub link_libs: Vec<String>,
    pub crashpad_libs: Vec<String>, // Crashpad static libraries to link
    pub frameworks: Vec<String>,    // iOS/macOS only

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
        let third_party = manifest_dir.join("third_party");
        let crashpad_dir = third_party.join("crashpad");

        let mut config = Self {
            target: target.clone(),
            profile: profile.clone(),
            out_dir,
            manifest_dir,
            crashpad_dir,
            compiler: PathBuf::from("c++"),
            archiver: "ar".to_string(),
            cxx_flags: vec!["-std=c++17".to_string()],
            gn_args: HashMap::new(),
            link_libs: vec!["stdc++".to_string(), "pthread".to_string()],
            crashpad_libs: vec![
                "crashpad_wrapper".to_string(),
                "client".to_string(),
                "common".to_string(),
                "util".to_string(),
                "format".to_string(),
                "minidump".to_string(),
                "snapshot".to_string(),
                "context".to_string(),
                "base".to_string(),
            ],
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

        // Disable tests (even though it shows a warning, it still works)
        config
            .gn_args
            .insert("crashpad_build_tests".to_string(), "false".to_string());

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
            return Err(format!("Unsupported target: {target}. Supported targets: android, ios, darwin, windows-msvc, linux").into());
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
            return Err(format!(
                "Unsupported Android architecture: {target}. Supported: aarch64, x86_64, armv7, i686"
            )
            .into());
        };

        // Compiler path (dynamic, not hardcoded)
        // Detect host platform for NDK prebuilt directory
        let ndk_host = if cfg!(target_os = "macos") {
            "darwin-x86_64"
        } else if cfg!(target_os = "linux") {
            "linux-x86_64"
        } else if cfg!(target_os = "windows") {
            "windows-x86_64"
        } else {
            return Err(
                "Unsupported host platform for Android NDK. Supported: macOS, Linux, Windows"
                    .into(),
            );
        };

        self.compiler = ndk
            .join(format!("toolchains/llvm/prebuilt/{ndk_host}/bin"))
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

        // Disable code signing for static library builds (CI environment)
        self.gn_args
            .insert("ios_enable_code_signing".to_string(), "false".to_string());

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
        self.crashpad_libs.push("mig_output".to_string()); // Crashpad iOS needs MIG-generated code
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
        self.crashpad_libs.push("mig_output".to_string()); // macOS needs MIG-generated code
        self.frameworks = vec![
            "Foundation".to_string(),
            "Security".to_string(),
            "CoreFoundation".to_string(),
            "IOKit".to_string(),
        ];
    }

    /// Configure for Windows
    ///
    /// Windows build configuration is split:
    /// - GN args: Only used for vendored build (not depot_tools)
    /// - Compiler/Link settings: Always used for wrapper and linking
    fn setup_windows(&mut self, target: &str) -> Result<(), Box<dyn std::error::Error>> {
        if !target.contains("msvc") {
            return Err(
                "Only MSVC target is supported for Windows. MinGW is not supported because Crashpad requires Windows SDK features."
                    .into(),
            );
        }

        let arch = if target.starts_with("x86_64") {
            "x64"
        } else if target.starts_with("i686") {
            "x86"
        } else {
            return Err(format!(
                "Unsupported Windows architecture: {target}. Supported: x86_64, i686"
            )
            .into());
        };

        // ===== GN build configuration (only for vendored build) =====
        self.gn_args
            .insert("target_os".to_string(), "\"win\"".to_string());
        self.gn_args
            .insert("target_cpu".to_string(), format!("\"{arch}\""));

        // Use dynamic CRT (/MD) to match Rust's default
        // This prevents LNK2038 runtime library mismatch errors
        self.gn_args
            .insert("is_component_build".to_string(), "false".to_string());
        self.gn_args
            .insert("use_custom_libcxx".to_string(), "false".to_string());

        // Force dynamic runtime
        if self.profile == "release" {
            self.gn_args
                .insert("extra_cflags".to_string(), "\"/MD\"".to_string());
        } else {
            self.gn_args
                .insert("extra_cflags".to_string(), "\"/MDd\"".to_string());
        }

        // ===== Compiler configuration (for wrapper compilation) =====
        // The cc crate will automatically find MSVC
        self.compiler = PathBuf::from("cl.exe");
        self.archiver = "lib".to_string();
        
        // CC flags for wrapper compilation
        // Note: cc crate handles most flags automatically
        self.cxx_flags = vec![];

        // ===== Linking configuration (always needed) =====
        // Windows-specific system libraries
        self.link_libs = vec![
            "advapi32".to_string(),
            "kernel32".to_string(),
            "user32".to_string(),
            "winmm".to_string(),
        ];
        
        // Crashpad libraries (same for all build strategies)
        self.crashpad_libs = vec![
            "client".to_string(),
            "common".to_string(),
            "util".to_string(),
            "base".to_string(),
        ];

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

        Err("Android NDK not found. Please set ANDROID_NDK_HOME environment variable or install cargo-ndk (cargo install cargo-ndk)".into())
    }

    /// Get build directory for current platform
    /// Uses a fixed path without hash for consistency between vendored and prebuild
    pub fn build_dir(&self) -> PathBuf {
        // Use fixed path: target/{target}/{profile}/crashpad_build
        self.manifest_dir
            .parent()
            .expect("Failed to get parent directory")
            .join("target")
            .join(&self.target)
            .join(&self.profile)
            .join("crashpad_build")
    }

    /// Get bindings output path
    pub fn bindings_path(&self) -> PathBuf {
        self.out_dir.join("bindings.rs")
    }
}

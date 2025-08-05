//! Platform-specific configuration and build logic
//!
//! This module provides a Rust-idiomatic way to handle platform differences
//! using enum-based strategy pattern.

use std::env;
use std::fmt;
use std::path::PathBuf;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq)]
pub enum Platform {
    Linux { arch: Arch },
    MacOS { arch: Arch },
    Ios { arch: Arch, simulator: bool },
    Android { arch: Arch, ndk_path: PathBuf },
    Windows { arch: Arch, msvc: bool },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Arch {
    X64,
    Arm64,
    Arm,
    X86,
}

#[derive(Debug)]
pub struct PlatformError(String);

impl fmt::Display for PlatformError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Platform error: {}", self.0)
    }
}

impl std::error::Error for PlatformError {}

impl FromStr for Arch {
    type Err = PlatformError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "x86_64" => Ok(Arch::X64),
            "aarch64" => Ok(Arch::Arm64),
            "arm" | "armv7" => Ok(Arch::Arm),
            "x86" => Ok(Arch::X86),
            _ => Err(PlatformError(format!("Unsupported architecture: {}", s))),
        }
    }
}

impl Arch {
    /// Convert to GN's CPU naming convention
    pub fn to_gn_cpu(&self) -> &'static str {
        match self {
            Arch::X64 => "x64",
            Arch::Arm64 => "arm64",
            Arch::Arm => "arm",
            Arch::X86 => "x86",
        }
    }
}

impl Platform {
    /// Detect the current platform from environment variables
    pub fn detect() -> Result<Self, PlatformError> {
        let target = env::var("TARGET")
            .map_err(|_| PlatformError("TARGET not set".to_string()))?;
        let os = env::var("CARGO_CFG_TARGET_OS")
            .map_err(|_| PlatformError("CARGO_CFG_TARGET_OS not set".to_string()))?;
        let arch_str = env::var("CARGO_CFG_TARGET_ARCH")
            .map_err(|_| PlatformError("CARGO_CFG_TARGET_ARCH not set".to_string()))?;
        let arch = Arch::from_str(&arch_str)?;

        match os.as_str() {
            "linux" => Ok(Platform::Linux { arch }),
            "macos" => Ok(Platform::MacOS { arch }),
            "ios" => Ok(Platform::Ios {
                arch,
                simulator: target.contains("sim"),
            }),
            "android" => {
                let ndk_path = env::var("ANDROID_NDK_HOME")
                    .map_err(|_| PlatformError("ANDROID_NDK_HOME not set for Android build".to_string()))?;
                Ok(Platform::Android {
                    arch,
                    ndk_path: PathBuf::from(ndk_path),
                })
            }
            "windows" => {
                let msvc = env::var("CARGO_CFG_TARGET_ENV")
                    .map(|env| env == "msvc")
                    .unwrap_or(false);
                Ok(Platform::Windows { arch, msvc })
            }
            _ => Err(PlatformError(format!("Unsupported OS: {}", os))),
        }
    }

    /// Get the build directory name for this platform
    pub fn build_name(&self) -> String {
        match self {
            Platform::Linux { arch } => format!("linux-{}", arch.to_gn_cpu()),
            Platform::MacOS { arch } => format!("macos-{}", arch.to_gn_cpu()),
            Platform::Ios { arch, simulator } => {
                if *simulator {
                    format!("ios-sim-{}", arch.to_gn_cpu())
                } else {
                    format!("ios-{}", arch.to_gn_cpu())
                }
            }
            Platform::Android { arch, .. } => format!("android-{}", arch.to_gn_cpu()),
            Platform::Windows { arch, .. } => format!("windows-{}", arch.to_gn_cpu()),
        }
    }

    /// Generate GN args for this platform
    pub fn gn_args(&self) -> Vec<String> {
        let mut args = vec!["is_debug=false".to_string()];

        match self {
            Platform::Linux { arch } => {
                args.push(format!("target_cpu=\"{}\"", arch.to_gn_cpu()));
            }
            Platform::MacOS { arch } => {
                args.push(format!("target_cpu=\"{}\"", arch.to_gn_cpu()));
            }
            Platform::Ios { arch, simulator } => {
                args.push("target_os=\"ios\"".to_string());
                args.push(format!("target_cpu=\"{}\"", arch.to_gn_cpu()));
                args.push("ios_enable_code_signing=false".to_string());
                if *simulator {
                    args.push("target_environment=\"simulator\"".to_string());
                    args.push("target_platform=\"iphoneos\"".to_string());
                }
            }
            Platform::Android { arch, ndk_path } => {
                args.push("target_os=\"android\"".to_string());
                args.push(format!("target_cpu=\"{}\"", arch.to_gn_cpu()));
                args.push(format!("android_ndk_root=\"{}\"", ndk_path.display()));
                args.push("android_api_level=21".to_string());
            }
            Platform::Windows { arch, .. } => {
                args.push(format!("target_cpu=\"{}\"", arch.to_gn_cpu()));
            }
        }

        args
    }

    /// Get compiler flags for this platform
    pub fn compile_flags(&self) -> Vec<&'static str> {
        let mut flags = vec!["-c", "-std=c++17"];

        match self {
            Platform::Linux { .. } => {
                flags.push("-fPIC");
            }
            Platform::MacOS { .. } => {
                flags.extend(&["-fPIC", "-mmacosx-version-min=10.9"]);
                flags.extend(&["-DCRASHPAD_MACOS", "-DOS_MACOSX=1"]);
            }
            Platform::Ios { simulator, .. } => {
                flags.push("-fPIC");
                flags.extend(&["-DCRASHPAD_IOS", "-DOS_IOS=1", "-DTARGET_OS_IOS=1"]);
                if *simulator {
                    flags.push("-mios-simulator-version-min=14.0");
                } else {
                    flags.push("-miphoneos-version-min=14.0");
                }
            }
            Platform::Android { .. } => {
                flags.push("-fPIC");
            }
            Platform::Windows { .. } => {}
        }

        flags
    }

    /// Get target triple for clang
    pub fn clang_target(&self) -> Option<&'static str> {
        match self {
            Platform::Ios { arch, simulator } => {
                if *simulator {
                    match arch {
                        Arch::Arm64 => Some("arm64-apple-ios14.0-simulator"),
                        Arch::X64 => Some("x86_64-apple-ios14.0-simulator"),
                        _ => None,
                    }
                } else {
                    match arch {
                        Arch::Arm64 => Some("arm64-apple-ios14.0"),
                        Arch::Arm => Some("armv7-apple-ios14.0"),
                        _ => None,
                    }
                }
            }
            _ => None,
        }
    }

    /// Get ninja targets for iOS (other platforms build default targets)
    pub fn ninja_targets(&self) -> Option<Vec<&'static str>> {
        match self {
            Platform::Ios { .. } => Some(vec![
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
            ]),
            _ => None,
        }
    }

    /// Get static libraries to link
    pub fn link_libraries(&self) -> Vec<&'static str> {
        let mut libs = vec![
            "crashpad_wrapper",
            "client",
            "common",
            "util",
        ];

        // MIG is only for macOS/iOS
        match self {
            Platform::MacOS { .. } | Platform::Ios { .. } => {
                libs.push("mig_output");
            }
            _ => {}
        }

        libs.extend(&[
            "format",
            "minidump",
            "snapshot",
            "context",
            "base",
        ]);

        libs
    }

    /// Get system libraries to link
    pub fn system_libraries(&self) -> Vec<&'static str> {
        match self {
            Platform::Linux { .. } => vec!["stdc++", "pthread"],
            Platform::MacOS { .. } => vec![
                "c++",
                "framework=Foundation",
                "framework=Security",
                "framework=CoreFoundation",
                "framework=IOKit",
                "dylib=bsm",
            ],
            Platform::Ios { .. } => vec![
                "c++",
                "z",
                "framework=Foundation",
                "framework=Security",
                "framework=CoreFoundation",
                "framework=UIKit",
            ],
            Platform::Android { .. } => vec!["c++_shared", "log"],
            Platform::Windows { .. } => vec![],
        }
    }

    /// Get the archiver tool for creating static libraries
    pub fn archiver(&self) -> &'static str {
        match self {
            Platform::MacOS { .. } | Platform::Ios { .. } => "libtool",
            _ => "ar",
        }
    }

    /// Check if this platform uses the in-process handler model
    pub fn is_in_process_handler(&self) -> bool {
        matches!(self, Platform::Ios { .. })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arch_from_str() {
        assert_eq!(Arch::from_str("x86_64").unwrap(), Arch::X64);
        assert_eq!(Arch::from_str("aarch64").unwrap(), Arch::Arm64);
        assert_eq!(Arch::from_str("armv7").unwrap(), Arch::Arm);
        assert!(Arch::from_str("unknown").is_err());
    }

    #[test]
    fn test_platform_build_name() {
        let linux = Platform::Linux { arch: Arch::X64 };
        assert_eq!(linux.build_name(), "linux-x64");

        let ios_sim = Platform::Ios {
            arch: Arch::Arm64,
            simulator: true,
        };
        assert_eq!(ios_sim.build_name(), "ios-sim-arm64");
    }

    #[test]
    fn test_link_libraries() {
        let linux = Platform::Linux { arch: Arch::X64 };
        let libs = linux.link_libraries();
        assert!(!libs.contains(&"mig_output"));

        let macos = Platform::MacOS { arch: Arch::Arm64 };
        let libs = macos.link_libraries();
        assert!(libs.contains(&"mig_output"));
    }
}
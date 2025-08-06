//! Platform detection and identification
//!
//! This module provides simple platform identification from environment variables.

use std::env;
use std::fmt;
use std::path::PathBuf;

/// Platform identification
#[derive(Debug, Clone, PartialEq)]
pub enum Platform {
    Linux { arch: Arch },
    MacOS { arch: Arch },
    Ios { arch: Arch, simulator: bool },
    Android { arch: Arch, ndk_path: PathBuf },
    Windows { arch: Arch, msvc: bool },
}

/// CPU architecture
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

impl Arch {
    /// Parse architecture from string
    pub fn from_str(s: &str) -> Result<Self, PlatformError> {
        match s {
            "x86_64" => Ok(Arch::X64),
            "aarch64" => Ok(Arch::Arm64),
            "arm" | "armv7" => Ok(Arch::Arm),
            "x86" | "i686" => Ok(Arch::X86),
            _ => Err(PlatformError(format!("Unsupported architecture: {s}"))),
        }
    }

    /// Convert to GN CPU name
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
    /// Detect platform from environment variables
    pub fn from_env() -> Result<Self, PlatformError> {
        let target = env::var("TARGET")
            .map_err(|_| PlatformError("TARGET environment variable not set".to_string()))?;

        let arch_str = target
            .split('-')
            .next()
            .ok_or_else(|| PlatformError("Invalid TARGET format".to_string()))?;
        let arch = Arch::from_str(arch_str)?;

        if target.contains("android") {
            // Try to find NDK path
            let ndk_path = env::var("ANDROID_NDK_HOME")
                .or_else(|_| env::var("ANDROID_NDK_ROOT"))
                .or_else(|_| env::var("NDK_HOME"))
                .map(PathBuf::from)
                .map_err(|_| PlatformError(
                    "Android target but NDK not found. Set ANDROID_NDK_HOME or ANDROID_NDK_ROOT".to_string()
                ))?;

            Ok(Platform::Android { arch, ndk_path })
        } else if target.contains("apple-ios") {
            let simulator = target.contains("sim");
            Ok(Platform::Ios { arch, simulator })
        } else if target.contains("apple") || target.contains("darwin") {
            Ok(Platform::MacOS { arch })
        } else if target.contains("windows") {
            let msvc = target.contains("msvc");
            Ok(Platform::Windows { arch, msvc })
        } else if target.contains("linux") {
            Ok(Platform::Linux { arch })
        } else {
            Err(PlatformError(format!("Unsupported target: {target}")))
        }
    }

    /// Get build directory name
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
}

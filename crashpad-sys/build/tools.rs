/// Binary tool management for GN and Ninja
///
/// This module handles downloading and caching of GN and Ninja binaries,
/// eliminating the need for depot_tools and Python dependencies.
use std::env;
use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::process::Command;

/// Tool versions from Crashpad's DEPS file
///
/// To update these versions:
/// 1. Open crashpad-sys/third_party/crashpad/DEPS
/// 2. Search for 'gn' in the deps_os section
/// 3. Copy the git_revision value for GN
/// 4. Search for 'ninja' in the deps section  
/// 5. Copy the version string for Ninja
/// 6. For Clang, look for 'windows/clang' and copy the object_name
///
/// Example from DEPS:
/// ```
/// 'buildtools/linux64': {
///     'packages': [
///         {
///             'package': 'gn/gn/linux-${{arch}}',
///             'version': 'git_revision:5e19d2fb166fbd4f6f32147fbb2f497091a54ad8',
///         }
///     ],
/// },
/// ```
///
/// These versions should be updated whenever Crashpad submodule is updated
/// to ensure compatibility with the build configuration.
const GN_VERSION: &str = "git_revision:5e19d2fb166fbd4f6f32147fbb2f497091a54ad8";
const NINJA_VERSION: &str = "version:2@1.8.2.chromium.3";

/// Manages build tool binaries (GN and Ninja)
pub struct BinaryToolManager {
    cache_dir: PathBuf,
    platform: Platform,
    verbose: bool,
}

#[derive(Debug, Clone)]
enum Platform {
    MacX64,
    MacArm64,
    LinuxX64,
    WinX64,
}

impl Platform {
    fn detect() -> Result<Self, Box<dyn std::error::Error>> {
        let os = env::consts::OS;
        let arch = env::consts::ARCH;

        match (os, arch) {
            ("macos", "x86_64") => Ok(Platform::MacX64),
            ("macos", "aarch64") => Ok(Platform::MacArm64),
            ("linux", "x86_64") => Ok(Platform::LinuxX64),
            ("windows", "x86_64") => Ok(Platform::WinX64),
            _ => Err(format!("Unsupported platform: {os}-{arch}").into()),
        }
    }

    fn gn_download_url(&self) -> String {
        let platform = match self {
            Platform::MacX64 => "mac-amd64",
            Platform::MacArm64 => "mac-arm64",
            Platform::LinuxX64 => "linux-amd64",
            Platform::WinX64 => "windows-amd64",
        };
        format!("https://chrome-infra-packages.appspot.com/dl/gn/gn/{platform}/+/{GN_VERSION}")
    }

    fn ninja_download_url(&self) -> String {
        let platform = match self {
            Platform::MacX64 => "mac-amd64",
            Platform::MacArm64 => "mac-arm64",
            Platform::LinuxX64 => "linux-amd64",
            Platform::WinX64 => "windows-amd64",
        };
        format!(
            "https://chrome-infra-packages.appspot.com/dl/infra/3pp/tools/ninja/{platform}/+/{NINJA_VERSION}"
        )
    }

    fn executable_suffix(&self) -> &str {
        match self {
            Platform::WinX64 => ".exe",
            _ => "",
        }
    }
}

impl BinaryToolManager {
    /// Create a new BinaryToolManager
    pub fn new(verbose: bool) -> Result<Self, Box<dyn std::error::Error>> {
        let platform = Platform::detect()?;

        // Platform-specific cache directory
        let platform_dir = format!("{}-{}", env::consts::OS, env::consts::ARCH);

        let cache_dir = if let Ok(dir) = env::var("CRASHPAD_CACHE_DIR") {
            PathBuf::from(dir).join("bin").join(&platform_dir)
        } else {
            dirs::cache_dir()
                .ok_or("Could not determine cache directory")?
                .join("crashpad-cache")
                .join("bin")
                .join(&platform_dir)
        };

        // Ensure cache directory exists
        fs::create_dir_all(&cache_dir)?;

        Ok(Self {
            cache_dir,
            platform,
            verbose,
        })
    }

    /// Ensure GN binary is available, downloading if necessary
    pub fn ensure_gn(&self) -> Result<PathBuf, Box<dyn std::error::Error>> {
        // Check if already in cache
        let gn_name = format!("gn{}", self.platform.executable_suffix());
        let cached_path = self.cache_dir.join(&gn_name);

        if cached_path.exists() {
            // Verify it's executable
            if let Ok(output) = Command::new(&cached_path).arg("--version").output() {
                if output.status.success() {
                    if self.verbose {
                        let version = String::from_utf8_lossy(&output.stdout);
                        eprintln!("Using cached GN (version: {})", version.trim());
                    }
                    return Ok(cached_path);
                }
            }
        }

        // Download GN
        if self.verbose {
            eprintln!("GN not found in cache, downloading...");
        }
        self.download_gn(&cached_path)?;
        Ok(cached_path)
    }

    /// Ensure Ninja binary is available, downloading if necessary
    pub fn ensure_ninja(&self) -> Result<PathBuf, Box<dyn std::error::Error>> {
        // Check if already in cache
        let ninja_name = format!("ninja{}", self.platform.executable_suffix());
        let cached_path = self.cache_dir.join(&ninja_name);

        if cached_path.exists() {
            // Verify it's executable
            if let Ok(output) = Command::new(&cached_path).arg("--version").output() {
                if output.status.success() {
                    if self.verbose {
                        let version = String::from_utf8_lossy(&output.stdout);
                        eprintln!("Using cached Ninja (version: {})", version.trim());
                    }
                    return Ok(cached_path);
                }
            }
        }

        // Download Ninja
        if self.verbose {
            eprintln!("Ninja not found in cache, downloading...");
        }
        self.download_ninja(&cached_path)?;
        Ok(cached_path)
    }

    /// Download GN binary from Chrome Infrastructure
    fn download_gn(&self, target_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        if self.verbose {
            eprintln!("Downloading GN binary...");
        }

        let url = self.platform.gn_download_url();
        let temp_zip = self.cache_dir.join("gn_temp.zip");

        // Download using ureq
        let response = ureq::get(&url).call()?;
        let mut reader = response.into_reader();
        let mut buffer = Vec::new();
        reader.read_to_end(&mut buffer)?;
        fs::write(&temp_zip, buffer)?;

        // Extract GN from zip
        let file = fs::File::open(&temp_zip)?;
        let mut archive = zip::ZipArchive::new(file)?;

        // Try to find and extract 'gn' binary
        let mut found = false;

        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let name = file.name();

            // Look for 'gn' or 'gn.exe' in the archive
            if name == "gn"
                || name == "gn.exe"
                || name.ends_with("/gn")
                || name.ends_with("/gn.exe")
            {
                let mut outfile = fs::File::create(target_path)?;
                io::copy(&mut file, &mut outfile)?;
                found = true;
                break;
            }
        }

        if !found {
            return Err("GN binary not found in archive".into());
        }

        // Clean up temp file
        let _ = fs::remove_file(&temp_zip);

        // Make executable on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(target_path)?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(target_path, perms)?;
        }

        if self.verbose {
            eprintln!("GN downloaded successfully to {}", target_path.display());
        }

        Ok(())
    }

    /// Download Ninja binary from GitHub
    fn download_ninja(&self, target_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        if self.verbose {
            eprintln!("Downloading Ninja binary...");
        }

        let url = self.platform.ninja_download_url();
        let temp_zip = self.cache_dir.join("ninja_temp.zip");

        // Download using ureq
        let response = ureq::get(&url).call()?;
        let mut reader = response.into_reader();
        let mut buffer = Vec::new();
        reader.read_to_end(&mut buffer)?;
        fs::write(&temp_zip, buffer)?;

        // Extract ninja from zip
        let file = fs::File::open(&temp_zip)?;
        let mut archive = zip::ZipArchive::new(file)?;

        // Extract all files (ninja releases are simple: just ninja binary + README)
        let mut extracted = false;
        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let name = file.name();

            // Look for ninja binary
            if name == "ninja" || name == "ninja.exe" {
                let mut outfile = fs::File::create(target_path)?;
                io::copy(&mut file, &mut outfile)?;
                extracted = true;
                break;
            }
        }

        if !extracted {
            return Err("Ninja binary not found in archive".into());
        }

        // Clean up temp file
        let _ = fs::remove_file(&temp_zip);

        // Make executable on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(target_path)?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(target_path, perms)?;
        }

        if self.verbose {
            eprintln!("Ninja downloaded successfully to {}", target_path.display());
        }

        Ok(())
    }
}

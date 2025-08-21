use anyhow::Result;
use xshell::{cmd, Shell};

use crate::utils::find_workspace_root;

pub fn clean(sh: &Shell) -> Result<()> {
    println!("Cleaning all build artifacts and caches...");

    let workspace_root = find_workspace_root(sh)?;
    
    // Clean Rust target
    println!("🧹 Running cargo clean...");
    cmd!(sh, "cargo clean").run()?;

    // Clean depot_tools from vendored-depot builds
    println!("🧹 Cleaning depot_tools cache...");
    let target_dir = workspace_root.join("target");
    if target_dir.exists() {
        // Find and remove all depot_tools directories
        for entry in std::fs::read_dir(&target_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                let depot_tools = path.join("depot_tools");
                if depot_tools.exists() {
                    sh.remove_path(&depot_tools)?;
                    println!("  ✓ Removed {}", depot_tools.display());
                }
                
                // Also clean crashpad_build directories
                for profile in &["debug", "release"] {
                    let crashpad_build = path.join(profile).join("crashpad_build");
                    if crashpad_build.exists() {
                        sh.remove_path(&crashpad_build)?;
                        println!("  ✓ Removed {}", crashpad_build.display());
                    }
                }
            }
        }
    }

    // Clean gclient dependencies in crashpad
    println!("🧹 Cleaning Crashpad third_party dependencies...");
    let crashpad_third_party = workspace_root
        .join("crashpad-sys")
        .join("third_party")
        .join("crashpad")
        .join("third_party");
    if crashpad_third_party.exists() {
        // Remove all subdirectories except .gitkeep files
        for entry in std::fs::read_dir(&crashpad_third_party)? {
            let entry = entry?;
            let path = entry.path();
            let filename = path.file_name().unwrap().to_string_lossy();
            if filename != ".gitkeep" && filename != ".git" {
                sh.remove_path(&path)?;
                println!("  ✓ Removed crashpad/third_party/{}", filename);
            }
        }
    }

    // Clean binary tool cache (GN, Ninja, depot_tools)
    println!("🧹 Cleaning binary tool cache...");
    let cache_cleaned = if cfg!(target_os = "macos") {
        let cache_dir = dirs::home_dir()
            .map(|h| h.join("Library/Caches/crashpad-cache"))
            .filter(|p| p.exists());
        
        if let Some(cache) = cache_dir {
            sh.remove_path(&cache)?;
            println!("  ✓ Removed {}", cache.display());
            true
        } else {
            false
        }
    } else if cfg!(target_os = "linux") {
        let cache_dir = dirs::cache_dir()
            .map(|c| c.join("crashpad-cache"))
            .filter(|p| p.exists());
        
        if let Some(cache) = cache_dir {
            sh.remove_path(&cache)?;
            println!("  ✓ Removed {}", cache.display());
            true
        } else {
            false
        }
    } else if cfg!(target_os = "windows") {
        let cache_dir = dirs::cache_dir()
            .map(|c| c.join("crashpad-cache"))
            .filter(|p| p.exists());
        
        if let Some(cache) = cache_dir {
            sh.remove_path(&cache)?;
            println!("  ✓ Removed {}", cache.display());
            true
        } else {
            false
        }
    } else {
        false
    };

    if !cache_cleaned {
        println!("  ℹ️  No cache directory found or unsupported platform");
    }

    // Clean dist directory if it exists
    let dist_dir = workspace_root.join("dist");
    if dist_dir.exists() {
        sh.remove_path(&dist_dir)?;
        println!("✓ Removed dist/");
    }

    println!("\n✅ Clean completed!");
    Ok(())
}
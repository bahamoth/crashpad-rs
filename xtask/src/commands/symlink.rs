use anyhow::Result;
use xshell::Shell;

#[cfg(unix)]
use crate::utils::find_workspace_root;
#[cfg(unix)]
use std::path::PathBuf;

pub fn create_symlinks(#[cfg_attr(windows, allow(unused_variables))] sh: &Shell) -> Result<()> {
    println!("🔗 Preparing for packaging...");

    // With vendored-depot, dependencies are handled automatically during build
    // This function is kept for backward compatibility with cargo package workflow

    #[cfg(unix)]
    {
        println!("Creating symlinks for Crashpad dependencies...");

        let deps = vec![
            ("mini_chromium", "mini_chromium"),
            ("googletest", "googletest"),
            ("zlib", "zlib"),
            ("libfuzzer", "src"),
            ("edo", "edo"),
            ("lss", "lss"),
        ];

        let workspace_root = find_workspace_root(sh)?;
        let crashpad_dir = workspace_root.join("crashpad-sys/third_party/crashpad");

        for (dep_name, subdir) in deps {
            let target = workspace_root.join(format!("crashpad-sys/third_party/{dep_name}"));
            let link = crashpad_dir.join("third_party").join(dep_name).join(subdir);

            // Skip if link already exists
            if link.exists() {
                println!("  ⏭️  {dep_name} already linked");
                continue;
            }

            // Skip if target doesn't exist
            if !target.exists() {
                println!("  ⚠️  {dep_name} source not found, skipping");
                continue;
            }

            // Create parent directory
            if let Some(parent) = link.parent() {
                sh.create_dir(parent)?;
            }

            // Calculate relative path from link to target
            let link_parent = link.parent().unwrap();
            let mut rel_path = PathBuf::new();

            // Count how many directories up we need to go
            let link_components: Vec<_> = link_parent
                .strip_prefix(&workspace_root)
                .unwrap_or(link_parent)
                .components()
                .collect();
            let target_components: Vec<_> = target
                .strip_prefix(&workspace_root)
                .unwrap_or(&target)
                .components()
                .collect();

            // Find common prefix length
            let common_len = link_components
                .iter()
                .zip(target_components.iter())
                .take_while(|(a, b)| a == b)
                .count();

            // Add ../ for each directory we need to go up
            for _ in common_len..link_components.len() {
                rel_path.push("..");
            }

            // Add the remaining target path
            for component in &target_components[common_len..] {
                rel_path.push(component);
            }

            use std::os::unix::fs::symlink;
            symlink(&rel_path, &link)?;
            println!("  ✓ Linked {} -> {}", dep_name, rel_path.display());
        }

        println!("✅ Symlinks created successfully");
    }

    #[cfg(windows)]
    {
        println!("ℹ️  Windows: Using vendored-depot for dependency management");
        println!("✅ Ready for packaging");
    }

    Ok(())
}

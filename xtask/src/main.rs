use anyhow::{Context, Result};
use chrono::Local;
use clap::{Parser, Subcommand};
use regex::Regex;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use xshell::{cmd, Shell};

#[derive(Parser)]
#[command(author, version, about = "Development tasks for crashpad-rs")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Build the project
    Build {
        /// Build in release mode
        #[arg(long)]
        release: bool,
    },
    /// Create a distribution package
    Dist {
        /// Target directory for distribution
        #[arg(long, default_value = "dist")]
        output: PathBuf,
    },
    /// Run tests
    Test,
    /// Clean build artifacts
    Clean,
    /// Install external development tools
    InstallTools,
    /// Update submodules to match Crashpad's DEPS
    UpdateDeps {
        /// Create a PR after updating (requires gh CLI)
        #[arg(long)]
        create_pr: bool,
    },
    /// Create symlinks for Crashpad dependencies (copy on Windows)
    Symlink,
    /// Copy dependencies for Windows packaging
    #[cfg(windows)]
    CopyDeps,
    /// Build prebuilt packages for distribution
    BuildPrebuilt {
        /// Target triple (optional, defaults to current)
        #[arg(long)]
        target: Option<String>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let sh = Shell::new()?;

    match cli.command {
        Commands::Build { release } => build(&sh, release)?,
        Commands::Dist { output } => dist(&sh, &output)?,
        Commands::Test => test(&sh)?,
        Commands::Clean => clean(&sh)?,
        Commands::InstallTools => install_tools(&sh)?,
        Commands::UpdateDeps { create_pr } => update_deps(&sh, create_pr)?,
        Commands::Symlink => create_symlinks(&sh)?,
        #[cfg(windows)]
        Commands::CopyDeps => copy_deps(&sh)?,
        Commands::BuildPrebuilt { target } => build_prebuilt(&sh, target)?,
    }

    Ok(())
}

fn build(sh: &Shell, release: bool) -> Result<()> {
    println!("Building crashpad-rs...");

    if release {
        cmd!(sh, "cargo build --release").run()?;
    } else {
        cmd!(sh, "cargo build").run()?;
    }

    println!("✅ Build completed successfully!");
    Ok(())
}

fn dist(sh: &Shell, output_dir: &Path) -> Result<()> {
    println!("Creating distribution package...");

    // Build in release mode first
    build(sh, true)?;

    // Create output directory structure
    sh.create_dir(output_dir)?;
    let lib_dir = output_dir.join("lib");
    let include_dir = output_dir.join("include");
    let bin_dir = output_dir.join("bin");

    sh.create_dir(&lib_dir)?;
    sh.create_dir(&include_dir)?;
    sh.create_dir(&bin_dir)?;

    // Find the workspace root
    let workspace_root = find_workspace_root(sh)?;

    // Detect platform
    let (os, arch) = detect_platform();
    let platform = format!("{os}-{arch}");

    // Handler executable name
    let handler_name = if cfg!(windows) {
        "crashpad_handler.exe"
    } else {
        "crashpad_handler"
    };

    // Find and copy the built handler
    let handler_path = workspace_root
        .join("third_party/crashpad_checkout/crashpad/out")
        .join(&platform)
        .join(handler_name);

    if !handler_path.exists() {
        anyhow::bail!(
            "crashpad_handler not found at: {}\nMake sure to build crashpad-rs-sys first",
            handler_path.display()
        );
    }

    let dist_handler = bin_dir.join(handler_name);
    sh.copy_file(&handler_path, &dist_handler)?;
    println!("✓ Copied crashpad_handler to dist/bin/");

    // Copy Rust libraries
    let target_dir = workspace_root.join("target/release");
    let lib_files = ["libcrashpad_rs.rlib", "libcrashpad_rs_sys.rlib"];

    for lib in &lib_files {
        let src = target_dir.join(lib);
        if src.exists() {
            let dest = lib_dir.join(lib);
            sh.copy_file(&src, &dest)?;
            println!("✓ Copied {lib} to dist/lib/");
        }
    }

    // Copy include files
    let sys_dir = workspace_root.join("crashpad-sys");
    let wrapper_h = sys_dir.join("wrapper.h");
    if wrapper_h.exists() {
        sh.copy_file(&wrapper_h, include_dir.join("wrapper.h"))?;
        println!("✓ Copied wrapper.h to dist/include/");
    }

    // Create README for the distribution
    let readme_content = format!(
        r#"# Crashpad Distribution Package

Platform: {platform}
Build: Release

## Contents

- `bin/` - Crashpad handler executable
- `lib/` - Rust libraries
- `include/` - Header files

## Usage

1. Set the handler path in your code:
   ```rust
   use crashpad::CrashpadConfig;

   let config = CrashpadConfig::new()
       .database_path("./crashes")
       .handler_path("./dist/bin/{handler_name}")
       .build();
   ```

2. Link the libraries in your Cargo.toml or build script.

For more information, see the main README.md in the repository.
"#
    );

    sh.write_file(output_dir.join("README.md"), readme_content)?;
    println!("✓ Created README.md");

    println!(
        "\n✅ Distribution package created at: {}",
        output_dir.display()
    );
    Ok(())
}

fn test(sh: &Shell) -> Result<()> {
    println!("Running tests...");

    // Run unit tests
    cmd!(sh, "cargo test --lib").run()?;

    // Run integration tests with nextest for process isolation
    cmd!(sh, "cargo nextest run --test '*'").run()?;

    println!("✅ All tests passed!");
    Ok(())
}

fn clean(sh: &Shell) -> Result<()> {
    println!("Cleaning build artifacts...");

    // Clean Rust target
    cmd!(sh, "cargo clean").run()?;

    // Clean native build artifacts
    let workspace_root = find_workspace_root(sh)?;
    let native_dirs = ["third_party/crashpad_checkout", "third_party/depot_tools"];

    for dir in &native_dirs {
        let path = workspace_root.join(dir);
        if path.exists() {
            sh.remove_path(&path)?;
            println!("✓ Removed {dir}");
        }
    }

    println!("✅ Clean completed!");
    Ok(())
}

fn install_tools(sh: &Shell) -> Result<()> {
    println!("Installing development tools...");

    // Install cargo-nextest for better test isolation
    println!("Installing cargo-nextest...");
    cmd!(sh, "cargo install cargo-nextest --locked").run()?;

    // Install cargo-ndk for Android cross-compilation
    println!("Installing cargo-ndk...");
    cmd!(sh, "cargo install cargo-ndk").run()?;

    println!("✅ Tools installed successfully!");
    Ok(())
}

fn find_workspace_root(sh: &Shell) -> Result<PathBuf> {
    let output = cmd!(sh, "cargo metadata --no-deps --format-version 1")
        .read()
        .context("Failed to get cargo metadata")?;

    let metadata: serde_json::Value = serde_json::from_str(&output)?;
    let root = metadata["workspace_root"]
        .as_str()
        .context("Failed to get workspace root")?;

    Ok(PathBuf::from(root))
}

fn detect_platform() -> (&'static str, &'static str) {
    let os = if cfg!(target_os = "macos") {
        "mac"
    } else if cfg!(target_os = "linux") {
        "linux"
    } else if cfg!(target_os = "windows") {
        "win"
    } else {
        "unknown"
    };

    let arch = if cfg!(target_arch = "x86_64") {
        "x64"
    } else if cfg!(target_arch = "aarch64") {
        "arm64"
    } else if cfg!(target_arch = "x86") {
        "x86"
    } else {
        "unknown"
    };

    (os, arch)
}

fn update_deps(sh: &Shell, create_pr: bool) -> Result<()> {
    println!("Updating submodules to match Crashpad's DEPS...");

    let workspace_root = find_workspace_root(sh)?;
    sh.change_dir(&workspace_root);

    // Step 1: Update crashpad to latest main
    println!("📦 Updating Crashpad to latest main...");
    sh.change_dir(&workspace_root);
    cmd!(
        sh,
        "git submodule update --init --remote crashpad-sys/third_party/crashpad"
    )
    .run()?;
    sh.change_dir(workspace_root.join("crashpad-sys/third_party/crashpad"));
    let crashpad_rev = cmd!(sh, "git rev-parse HEAD").read()?;
    println!("  Crashpad updated to: {}", crashpad_rev.trim());

    // Step 2: Parse DEPS file
    println!("📄 Parsing DEPS file...");
    let deps_path = workspace_root.join("crashpad-sys/third_party/crashpad/DEPS");
    let deps_content = sh.read_file(&deps_path)?;
    let deps = parse_deps(&deps_content)?;

    // Step 3: Skip .gitmodules update (no longer needed)
    // Submodules are tracked by their commit hash, not branch
    sh.change_dir(&workspace_root);

    // Step 4: Update each submodule to the specified revision
    println!("🔄 Updating submodules to specified revisions...");
    sh.change_dir(&workspace_root);
    for (name, rev) in &deps {
        if name == "crashpad" {
            continue; // Already updated
        }
        let submodule_path = format!("crashpad-sys/third_party/{name}");
        if workspace_root.join(&submodule_path).exists() {
            println!("  Updating {name} to {rev}");
            // First fetch the latest refs
            cmd!(sh, "git submodule update --init {submodule_path}").run()?;
            // Then checkout specific revision in submodule
            sh.change_dir(workspace_root.join(&submodule_path));
            cmd!(sh, "git fetch origin").run()?;
            cmd!(sh, "git checkout {rev}").run()?;
            sh.change_dir(&workspace_root);
            // Record the change in the parent repository
            cmd!(sh, "git add {submodule_path}").run()?;
        }
    }

    sh.change_dir(&workspace_root);

    // Step 5: Check for changes
    let status = cmd!(sh, "git status --porcelain").read()?;
    if status.is_empty() {
        println!("✅ No changes needed, already up to date!");
        return Ok(());
    }

    // Step 6: Show summary of changes
    println!("\n📋 Summary of changes:");
    cmd!(sh, "git diff --stat").run()?;

    if create_pr {
        // Step 7: Create branch and commit
        let date = Local::now().format("%Y%m%d").to_string();
        let branch_name = format!("auto/update-deps-{date}");

        println!("\n🌿 Creating branch: {branch_name}");
        cmd!(sh, "git checkout -b {branch_name}").run()?;

        println!("💾 Committing changes...");
        cmd!(sh, "git add -A").run()?;
        let commit_msg = format!("chore: update submodules to match Crashpad DEPS\n\nAutomatically updated submodules to match revisions in:\ncrashpad-sys/third_party/crashpad/DEPS @ {}", crashpad_rev.trim());
        cmd!(sh, "git commit -m {commit_msg}").run()?;

        println!("📤 Pushing branch...");
        cmd!(sh, "git push -u origin {branch_name}").run()?;

        println!("🔄 Creating pull request...");
        let pr_body = "This PR automatically updates submodules to match the revisions specified in Crashpad's DEPS file.\n\nGenerated by `cargo xtask update-deps --create-pr`";
        cmd!(sh, "gh pr create --title 'chore: update submodules to match Crashpad DEPS' --body {pr_body}").run()?;

        println!("✅ Pull request created successfully!");
    } else {
        println!("\n💡 To create a PR, run: cargo xtask update-deps --create-pr");
    }

    Ok(())
}

fn parse_deps(content: &str) -> Result<HashMap<String, String>> {
    let mut deps = HashMap::new();

    // Pattern to match dependencies in DEPS file
    // Format: 'crashpad/third_party/xxx/xxx': ... '@' + 'hash',
    let dep_pattern =
        Regex::new(r"'crashpad/third_party/([^/]+)/[^']+':.*?@'\s*\+\s*\n\s*'([a-f0-9]{40})'")?;

    for cap in dep_pattern.captures_iter(content) {
        let name = cap.get(1).unwrap().as_str();
        let rev = cap.get(2).unwrap().as_str();
        deps.insert(name.to_string(), rev.to_string());
    }

    // Add crashpad itself (current HEAD)
    deps.insert("crashpad".to_string(), "HEAD".to_string());

    Ok(deps)
}

fn create_symlinks(sh: &Shell) -> Result<()> {
    #[cfg(windows)]
    {
        println!("📁 Copying dependencies for Windows (symlinks not used)...");
    }
    #[cfg(unix)]
    {
        println!("🔗 Creating symlinks for Crashpad dependencies...");
    }

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

        // Create symlink or copy
        #[cfg(unix)]
        {
            use std::os::unix::fs::symlink;
            symlink(&rel_path, &link)?;
            println!("  ✓ Linked {} -> {}", dep_name, rel_path.display());
        }

        #[cfg(windows)]
        {
            // Windows: Always copy instead of symlink
            copy_dir_all(&target, &link)?;
            println!("  ✓ Copied {} to {}", dep_name, link.display());
        }
    }

    #[cfg(unix)]
    println!("✅ Symlinks created successfully");
    #[cfg(windows)]
    println!("✅ Dependencies copied successfully");

    println!("📦 You can now run: cargo package --package crashpad-rs-sys");

    Ok(())
}

/// Copy dependencies for Windows packaging
#[cfg(windows)]
fn copy_deps(sh: &Shell) -> Result<()> {
    println!("📁 Copying dependencies for Windows packaging...");

    let workspace_root = find_workspace_root(sh)?;
    let crashpad_dir = workspace_root.join("crashpad-sys/third_party/crashpad");

    let deps = vec![
        ("mini_chromium", "mini_chromium"),
        ("googletest", "googletest"),
        ("zlib", "zlib"),
        ("libfuzzer", "src"),
        ("edo", "edo"),
        ("lss", "lss"),
    ];

    for (dep_name, subdir) in deps {
        let src = workspace_root.join(format!("crashpad-sys/third_party/{}", dep_name));
        let dst = crashpad_dir.join("third_party").join(dep_name).join(subdir);

        if !src.exists() {
            println!("  ⚠️  {} source not found, skipping", dep_name);
            continue;
        }

        if dst.exists() {
            println!("  ⏭️  {} already exists, skipping", dep_name);
            continue;
        }

        copy_dir_all(&src, &dst)?;
        println!("  ✓ Copied {}", dep_name);
    }

    println!("✅ Dependencies copied successfully");
    println!("📦 You can now run: cargo package --package crashpad-rs-sys");

    Ok(())
}

/// Helper function to copy directory recursively
#[cfg(windows)]
fn copy_dir_all(src: &Path, dst: &Path) -> Result<()> {
    use std::fs;

    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let file_name = entry.file_name();
        let dst_path = dst.join(&file_name);

        if src_path.is_dir() {
            copy_dir_all(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}

/// Build prebuilt packages for distribution
fn build_prebuilt(sh: &Shell, target: Option<String>) -> Result<()> {
    println!("🔨 Building prebuilt package...");

    let workspace_root = find_workspace_root(sh)?;
    sh.change_dir(&workspace_root);

    // Get target triple
    let target = target.unwrap_or_else(|| {
        std::env::var("TARGET").unwrap_or_else(|_| {
            // Detect current platform
            let output = std::process::Command::new("rustc")
                .args(&["-vV"])
                .output()
                .expect("Failed to get rustc version");
            let output_str = String::from_utf8_lossy(&output.stdout);

            // Extract host triple from rustc output
            for line in output_str.lines() {
                if line.starts_with("host:") {
                    return line.split_whitespace().nth(1).unwrap().to_string();
                }
            }

            panic!("Could not determine target triple");
        })
    });

    println!("📦 Target: {}", target);

    // Build crashpad using vendored-depot
    println!("🏗️  Building crashpad with vendored-depot...");
    cmd!(sh, "cargo build --package crashpad-rs-sys --no-default-features --features vendored-depot --target {target}").run()?;

    // Find OUT_DIR from build
    let out_dir = find_out_dir(sh, &target)?;
    println!("📂 Build output: {}", out_dir.display());

    // Create dist directory structure
    let dist_dir = workspace_root.join("dist").join(&target);
    sh.create_dir(&dist_dir)?;
    let lib_dir = dist_dir.join("lib");
    let include_dir = dist_dir.join("include");
    sh.create_dir(&lib_dir)?;
    sh.create_dir(&include_dir)?;

    // Copy libraries
    println!("📚 Copying libraries...");
    let lib_patterns = if target.contains("windows") {
        vec!["*.lib"]
    } else {
        vec!["*.a"]
    };

    for pattern in &lib_patterns {
        let files = std::fs::read_dir(&out_dir)?
            .filter_map(|entry| entry.ok())
            .filter(|entry| {
                let name = entry.file_name();
                let name_str = name.to_string_lossy();
                name_str.ends_with(pattern.trim_start_matches('*'))
            });

        for file in files {
            let src = file.path();
            let dst = lib_dir.join(file.file_name());
            std::fs::copy(&src, &dst)?;
            println!("  ✓ {}", file.file_name().to_string_lossy());
        }
    }

    // Copy crashpad_handler
    let handler_name = if target.contains("windows") {
        "crashpad_handler.exe"
    } else {
        "crashpad_handler"
    };

    let handler_src = out_dir.join(handler_name);
    if handler_src.exists() {
        let handler_dst = dist_dir.join(handler_name);
        std::fs::copy(&handler_src, &handler_dst)?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&handler_dst)?.permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(&handler_dst, perms)?;
        }

        println!("  ✓ {}", handler_name);
    }

    // Copy wrapper.h
    let wrapper_src = workspace_root.join("crashpad-sys/wrapper.h");
    let wrapper_dst = include_dir.join("wrapper.h");
    std::fs::copy(&wrapper_src, &wrapper_dst)?;
    println!("  ✓ wrapper.h");

    // Get package version
    let cargo_toml = workspace_root.join("crashpad-sys/Cargo.toml");
    let toml_content = std::fs::read_to_string(&cargo_toml)?;
    let version = toml_content
        .lines()
        .find(|line| line.starts_with("version"))
        .and_then(|line| line.split('=').nth(1))
        .map(|v| v.trim().trim_matches('"'))
        .unwrap_or("unknown");

    // Create tarball
    let archive_name = format!("crashpad-{}-{}.tar.gz", version, target);
    let archive_path = workspace_root.join("dist").join(&archive_name);

    println!("📦 Creating archive: {}", archive_name);

    #[cfg(unix)]
    {
        sh.change_dir(&workspace_root.join("dist"));
        cmd!(sh, "tar czf {archive_name} {target}").run()?;
    }

    #[cfg(windows)]
    {
        // Use PowerShell to create archive on Windows
        let ps_script = format!(
            "Compress-Archive -Path '{}' -DestinationPath '{}' -Force",
            dist_dir.display(),
            archive_path.display()
        );
        cmd!(sh, "powershell -Command {ps_script}").run()?;
    }

    // Generate checksum
    println!("🔐 Generating checksum...");
    let archive_content = std::fs::read(&archive_path)?;
    let digest = sha256::digest(&archive_content[..]);
    let checksum_path = archive_path.with_extension("tar.gz.sha256");
    std::fs::write(&checksum_path, format!("{}  {}\n", digest, archive_name))?;

    println!("\n✅ Prebuilt package created:");
    println!("  📦 {}", archive_path.display());
    println!("  🔐 {}", checksum_path.display());
    println!("\n📤 Ready to upload to GitHub Releases!");

    Ok(())
}

/// Find the OUT_DIR for a specific target
fn find_out_dir(sh: &Shell, target: &str) -> Result<PathBuf> {
    let workspace_root = find_workspace_root(sh)?;

    // Common OUT_DIR patterns
    let candidates = vec![
        workspace_root.join(format!("target/{}/debug/build", target)),
        workspace_root.join(format!("target/{}/release/build", target)),
        workspace_root.join("target/debug/build"),
        workspace_root.join("target/release/build"),
    ];

    for candidate in candidates {
        if candidate.exists() {
            // Find crashpad-rs-sys build directory
            for entry in std::fs::read_dir(&candidate)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() && path.to_string_lossy().contains("crashpad-rs-sys") {
                    let out_dir = path.join("out");
                    if out_dir.exists() {
                        return Ok(out_dir);
                    }
                }
            }
        }
    }

    anyhow::bail!(
        "Could not find OUT_DIR for target {}. Make sure the build completed successfully.",
        target
    )
}

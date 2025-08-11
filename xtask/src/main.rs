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
            "crashpad_handler not found at: {}\nMake sure to build crashpad-sys first",
            handler_path.display()
        );
    }

    let dist_handler = bin_dir.join(handler_name);
    sh.copy_file(&handler_path, &dist_handler)?;
    println!("✓ Copied crashpad_handler to dist/bin/");

    // Copy Rust libraries
    let target_dir = workspace_root.join("target/release");
    let lib_files = ["libcrashpad.rlib", "libcrashpad_sys.rlib"];

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

// No longer needed - submodules are tracked by commit hash, not branch
// fn update_gitmodules(sh: &Shell, deps: &HashMap<String, String>, crashpad_rev: &str) -> Result<()> {
//     // Removed - we don't use branch field in .gitmodules anymore
// }

mod commands;
mod utils;

use anyhow::Result;
use clap::{Parser, Subcommand};
use xshell::Shell;

use commands::{build, build_prebuilt, create_symlinks, dist, install_tools, test, update_deps};

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
    /// Package the crates for distribution
    Dist,
    /// Run tests in parallel using multiple processes
    Test,
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
    /// Build prebuilt packages for distribution
    BuildPrebuilt {
        /// Target triple (optional, defaults to current)
        #[arg(long)]
        target: Option<String>,
    },
}

fn main() -> Result<()> {
    // Parse CLI args, but handle the case where no command is provided
    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(0);
        }
    };

    let sh = Shell::new()?;

    match cli.command {
        Commands::Build { release } => build(&sh, release)?,
        Commands::Dist => dist(&sh)?,
        Commands::Test => test(&sh)?,
        Commands::InstallTools => install_tools(&sh)?,
        Commands::UpdateDeps { create_pr } => update_deps(&sh, create_pr)?,
        Commands::Symlink => create_symlinks(&sh)?,
        Commands::BuildPrebuilt { target } => build_prebuilt(&sh, target)?,
    }

    Ok(())
}

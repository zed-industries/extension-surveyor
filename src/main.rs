use std::path::PathBuf;

use anyhow::{Context as _, Result};
use clap::{Parser, Subcommand};
use tokio::fs;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Updates the extensions repository with the latest changes.
    UpdateRepo,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let extension_repository_url = "https://github.com/zed-industries/extensions.git";
    let work_dir = PathBuf::from("work");

    match cli.command {
        Command::UpdateRepo => {
            fs::create_dir_all(&work_dir).await?;

            tokio::process::Command::new("git")
                .args(["clone", "--recurse-submodules", &extension_repository_url])
                .arg(&work_dir)
                .spawn()?
                .wait()
                .await
                .context("failed to clone extensions repository")?;

            Ok(())
        }
    }
}

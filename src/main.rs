use std::path::PathBuf;

use anyhow::{Context as _, Result};
use clap::Parser;
use tokio::fs;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {}

#[tokio::main]
async fn main() -> Result<()> {
    let _cli = Cli::parse();

    let extension_repository_url = "https://github.com/zed-industries/extensions.git";
    let work_dir = PathBuf::from("work");

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

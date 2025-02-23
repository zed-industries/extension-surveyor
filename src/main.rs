mod extensions;
mod surveys;

use std::path::PathBuf;

use anyhow::{Context as _, Result};
use clap::{Args, Parser, Subcommand};
use surveys::ThemesUsingProperty;
use tokio::fs;

use crate::extensions::ExtensionsToml;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Runs a survey.
    Survey(SurveyArgs),
    /// Updates the extensions repository with the latest changes.
    UpdateRepo,
}

#[derive(Debug, Args)]
struct SurveyArgs {
    #[command(subcommand)]
    command: SurveyCommand,
}

#[derive(Debug, Subcommand)]
enum SurveyCommand {
    ThemeProperty {
        /// The name of the theme property to survey.
        name: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let extension_repository_url = "https://github.com/zed-industries/extensions.git";
    let work_dir = PathBuf::from("work");

    match cli.command {
        Command::Survey(survey) => {
            let extensions_toml = ExtensionsToml::load(&work_dir).await?;

            match survey.command {
                SurveyCommand::ThemeProperty { name } => {
                    let survey = ThemesUsingProperty::new(name);
                    survey.run(&work_dir, &extensions_toml).await?;

                    Ok(())
                }
            }
        }
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

mod cli;
mod extensions;
mod github;
mod survey;
mod surveys;

use std::path::PathBuf;

use anyhow::{Context as _, Result};
use clap::Parser as _;
use tokio::fs;

use crate::cli::{Cli, SurveyCommand};
use crate::extensions::ExtensionsToml;
use crate::survey::Survey as _;
use crate::surveys::{ExtensionJsonUsage, ThemePropertyUsage, TreeSitterGrammars};

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let extension_repository_url = "https://github.com/zed-industries/extensions.git";
    let work_dir = PathBuf::from("work");

    match cli.command {
        cli::Command::Survey(survey) => {
            let extensions_toml = ExtensionsToml::load(&work_dir).await?;

            match survey.command {
                SurveyCommand::ThemeProperty { name } => {
                    let survey = ThemePropertyUsage::new(name);
                    survey.run(&work_dir, &extensions_toml).await?;

                    Ok(())
                }
                SurveyCommand::ExtensionJson => {
                    let survey = ExtensionJsonUsage;
                    survey.run(&work_dir, &extensions_toml).await?;

                    Ok(())
                }
                SurveyCommand::TreeSitterGrammars => {
                    let survey = TreeSitterGrammars;
                    survey.run(&work_dir, &extensions_toml).await?;

                    Ok(())
                }
            }
        }
        cli::Command::UpdateRepo => {
            if work_dir.exists() {
                tokio::process::Command::new("git")
                    .args(["pull", "--recurse-submodules"])
                    .current_dir(&work_dir)
                    .spawn()?
                    .wait()
                    .await
                    .context("failed to pull extensions repository")?;
            } else {
                fs::create_dir_all(&work_dir).await?;

                tokio::process::Command::new("git")
                    .args(["clone", "--recurse-submodules", extension_repository_url])
                    .arg(&work_dir)
                    .spawn()?
                    .wait()
                    .await
                    .context("failed to clone extensions repository")?;
            }

            Ok(())
        }
    }
}

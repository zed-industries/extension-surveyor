use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[clap(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Runs a survey.
    Survey(SurveyArgs),
    /// Updates the extensions repository with the latest changes.
    UpdateRepo,
}

#[derive(Debug, Args)]
pub struct SurveyArgs {
    #[command(subcommand)]
    pub command: SurveyCommand,
}

#[derive(Debug, Subcommand)]
pub enum SurveyCommand {
    ThemeProperty {
        /// The name of the theme property to survey.
        name: String,
    },
}

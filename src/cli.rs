use clap::{ArgAction, Parser, Subcommand};

use crate::config::GLOBAL_DEFAULT_GROUP_NAME;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
pub struct Cli {
    #[clap(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Run a program
    #[command(
        visible_alias = "r",
        args_conflicts_with_subcommands = true,
        allow_external_subcommands = true
    )]
    Run(RunCommand),

    /// Add a new executable to a group
    #[command()]
    Add(AddCommand),

    /// List all groups and their executables
    #[command(visible_alias = "ls")]
    List(ListCommand),

    /// Initialize the configuration file
    #[command()]
    Init(InitCommand),

    /// Remove an executable from a group
    #[command()]
    Rm(RmCommand),
}

#[derive(Parser)]
pub struct RunCommand {
    /// The name of group
    #[arg(short='g', long="group", default_value=GLOBAL_DEFAULT_GROUP_NAME)]
    pub group: String,
    /// The arguments to pass to the program
    #[arg(action=ArgAction::Append, allow_negative_numbers = true, required = true, allow_hyphen_values = true)]
    pub args: Option<Vec<String>>,
}

#[derive(Parser)]
pub struct AddCommand {
    /// The name of group
    #[arg(short='g', long="group", default_value=GLOBAL_DEFAULT_GROUP_NAME)]
    pub group: String,
    /// The path to the executable
    #[arg()]
    pub path: String,

    /// alias for the executable
    #[arg(short='n', long="name")]
    pub name: Option<String>,
}

#[derive(Parser)]
pub struct ListCommand {
    /// Show all groups and their executables
    #[arg(short='a', long="all", action=ArgAction::SetTrue)]
    pub all: bool,

    /// The name of group
    #[arg(short='g', long="group", default_value=GLOBAL_DEFAULT_GROUP_NAME)]
    pub group: String,
}

#[derive(Parser)]
pub struct InitCommand {
    /// Force re-initialize, overwriting existing config file
    #[arg(short='f', long="force", action=ArgAction::SetTrue)]
    pub force: bool,
}

#[derive(Parser)]
pub struct RmCommand {
    /// The name of group
    #[arg(short='g', long="group", default_value=GLOBAL_DEFAULT_GROUP_NAME)]
    pub group: String,
    /// The name of the executable to remove
    #[arg(short = 'n', long = "name")]
    pub name: Option<String>,
}

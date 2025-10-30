use clap::{ArgAction, Parser, Subcommand};

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

    /// Switch to a group
    #[command(visible_alias = "s")]
    #[command()]
    Switch(SwitchCommand),

    /// Rename an executable alias
    #[command()]
    Rename(RenameCommand),

    /// Show detailed information about an executable
    #[command()]
    Info(InfoCommand),

    /// Enable an executable
    #[command()]
    Enable(EnableCommand),

    /// Disable an executable
    #[command()]
    Disable(DisableCommand),

    /// Search for executables
    #[command()]
    Search(SearchCommand),
}

#[derive(Parser)]
pub struct RunCommand {
    /// The name of group
    #[arg(short = 'g', long = "group")]
    pub group: Option<String>,
    /// The arguments to pass to the program
    #[arg(action=ArgAction::Append, allow_negative_numbers = true, required = true, allow_hyphen_values = true)]
    pub args: Option<Vec<String>>,
}

#[derive(Parser)]
pub struct AddCommand {
    /// The name of group
    #[arg(short = 'g', long = "group")]
    pub group: Option<String>,
    /// The path to the executable
    #[arg()]
    pub path: String,

    /// alias for the executable
    #[arg(short = 'n', long = "name")]
    pub name: Option<String>,
}

#[derive(Parser)]
pub struct ListCommand {
    /// Show all groups
    #[arg(short = 'a', long = "all", action=ArgAction::SetTrue)]
    pub all: bool,
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
    #[arg(short = 'g', long = "group")]
    pub group: Option<String>,
    /// The name of the executable to remove
    #[arg(short = 'n', long = "name")]
    pub name: Option<String>,
    /// Hard delete
    #[arg(short='d', long="delete", action=ArgAction::SetTrue)]
    pub delete: bool,
}

#[derive(Parser)]
pub struct SwitchCommand {
    /// The name of group
    pub group: Option<String>,
}

#[derive(Parser)]
pub struct RenameCommand {
    /// The name of group
    #[arg(short = 'g', long = "group")]
    pub group: Option<String>,
    /// The current name of the executable
    pub old_name: String,
    /// The new name for the executable
    pub new_name: String,
}

#[derive(Parser)]
pub struct InfoCommand {
    /// The name of group
    #[arg(short = 'g', long = "group")]
    pub group: Option<String>,
    /// The name of the executable
    pub name: String,
}

#[derive(Parser)]
pub struct EnableCommand {
    /// The name of group
    #[arg(short = 'g', long = "group")]
    pub group: Option<String>,
    /// The name of the executable
    pub name: String,
}

#[derive(Parser)]
pub struct DisableCommand {
    /// The name of group
    #[arg(short = 'g', long = "group")]
    pub group: Option<String>,
    /// The name of the executable
    pub name: String,
}

#[derive(Parser)]
pub struct SearchCommand {
    /// Search query (matches against executable name or path)
    pub query: String,
}

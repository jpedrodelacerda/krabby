use clap::{Args, Subcommand};

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Goes to project path
    Cd { project_name: String },
    /// Manage project hook
    Hook(Hook),
    /// Executes given string
    #[clap(visible_alias = "r")]
    Run { script: String },
    /// Manage projects on database
    Project(Project),
    /// Manage scripts of a project
    Script(Script),
    /// Print the helper script
    Shell(Shell),
}

#[derive(Debug, Args)]
#[command(args_conflicts_with_subcommands = true)]
pub struct Hook {
    #[command(subcommand)]
    pub command: Option<HookCommands>,
}

#[derive(Debug, Subcommand)]
pub enum HookCommands {
    /// Define hook to execute whenever you enter the project
    Set {
        #[clap(value_delimiter = ',')]
        hook: Vec<String>,
    },
}

#[derive(Debug, Args)]
#[command(args_conflicts_with_subcommands = true)]
pub struct Shell {
    #[command(subcommand)]
    pub command: Option<ShellCommands>,
}

#[derive(Debug, Subcommand)]
pub enum ShellCommands {
    /// Print `kb` function
    #[clap(visible_alias = "sh")]
    Bash,
    Zsh,
}

#[derive(Debug, Args)]
#[command(args_conflicts_with_subcommands = true)]
pub struct Script {
    #[command(subcommand)]
    pub command: Option<ScriptCommands>,
}

#[derive(Debug, Subcommand)]
pub enum ScriptCommands {
    /// Add script to Krabby project file
    Add {
        /// Script name to be registered
        script_name: String,
        /// Script command to be executed
        script_command: String,
    },
    /// Remove script to Krabby project file
    #[clap(visible_alias = "rm")]
    Remove { script: String },
    /// List available scripts
    #[clap(visible_alias = "ls")]
    List,
}

#[derive(Debug, Args)]
#[command(args_conflicts_with_subcommands = true)]
pub struct Project {
    #[command(subcommand)]
    pub command: ProjectCommands,
}

#[derive(Debug, Subcommand)]
pub enum ProjectCommands {
    /// Add project to Krabby database
    Add {
        /// Project name to be registered
        project_name: String,
        /// Project path
        project_path: String,
    },
    /// Go to project directory
    #[clap(visible_alias = "go")]
    Cd { project_name: String },
    /// List all registered projects
    #[clap(visible_alias = "ls")]
    List,
    /// Remove project from Krabby database
    #[clap(visible_alias = "rm")]
    Remove { project_name: String },
    /// Initialize a krabby project file
    Init {
        /// Specify the name of the project (defaults to directory name)
        project_name: Option<String>,
    },
}

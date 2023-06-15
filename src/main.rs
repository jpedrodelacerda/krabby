use std::{io::Error, path::PathBuf, process::exit};

use clap::Parser;
use krabby::{
    commands::*,
    database::Database,
    hook::ProjectHook,
    messages::Message,
    project::{self, ProjectName},
    script,
    shell::Flavor,
};
use owo_colors::OwoColorize;

#[derive(Debug, Parser)]
#[command(name = "krabby")]
#[command(version = clap::crate_version!())]
#[command(about = "A nice little project manager")]
struct Cli {
    /// Specify krabby.db file
    #[arg(global = true, short, long, value_name = "DATABASE")]
    database: Option<PathBuf>,

    /// Specify krabby.toml to act
    #[arg(global = true, short = 'f', long, value_name = "PROJECT_FILE")]
    project_file: Option<PathBuf>,

    #[command(subcommand)]
    command: Commands,
}

fn main() -> Result<(), Error> {
    let cli = Cli::parse();

    let database_path = cli
        .database
        .or(Some(dirs::home_dir().unwrap().join(".krabby.db")));
    if let Some(ref db) = database_path {
        if !db.exists() {
            let new_database = Database::new(Some(db.to_path_buf()));
            new_database.write().expect("Failed to write database");
        }
    }

    let project_file_path = cli.project_file.or(Some(PathBuf::from("./krabby.toml")));

    match cli.command {
        Commands::Cd { project_name } => {
            let project_name = project::ProjectName::parse(project_name);
            let database = Database::from_file(database_path.unwrap())
                .expect("Failed to read krabby database.");
            match database.go_to_project(&project_name) {
                Ok(_) => {
                    println!("echo \"You're good to go!\"");
                    exit(0);
                }
                Err(e) => {
                    println!("{}", e);
                    exit(1);
                }
            }
        }
        Commands::Project(project) => {
            let project_cmd = project.command;
            match project_cmd {
                ProjectCommands::Add {
                    project_name,
                    project_path,
                } => {
                    let mut database = Database::from_file(database_path.unwrap())
                        .expect("Failed to read krabby database.");
                    let project_name = ProjectName::parse(project_name);
                    match database.add_project(project_name.clone(), project_path.into()) {
                        Ok(_) => {
                            println!("{}", Message::RegisterProjectSuccess(project_name));
                            database.save();
                            exit(0);
                        }
                        Err(e) => {
                            println!("{}", Message::RegisterProjectFail(project_name, e));
                            exit(1);
                        }
                    }
                }
                ProjectCommands::List => {
                    let database = Database::from_file(database_path.unwrap())
                        .expect("Failed to read krabby database.");
                    if database.projects.is_empty() {
                        println!(
                            "So... it looks like you have no project registered yet.\n{}",
                            Message::RegisterProject
                        );
                        exit(0);
                    }
                    println!("So, let's take a look at your projects!");
                    for (name, projects) in database.projects {
                        println!(
                            "{} at {}",
                            name.bold(),
                            projects.into_os_string().to_str().unwrap().bold()
                        );
                    }
                    println!("And that's a wrap!");
                    exit(0);
                }
                ProjectCommands::Remove { project_name } => {
                    let mut database = Database::from_file(database_path.unwrap())
                        .expect("Failed to read krabby database");
                    match database.remove_project(ProjectName::parse(project_name.clone())) {
                        Ok(_) => {
                            println!(
                                "Project {} was removed from the database.",
                                project_name.bold()
                            );
                            database.save();
                            exit(0);
                        }
                        Err(e) => {
                            println!("Failed to remove project!\n{}", e);
                            exit(1);
                        }
                    }
                }
                ProjectCommands::Init { project_name } => {
                    let new_project_name = project_name
                        .or_else(|| {
                            let pwd = std::env::current_dir()
                                .expect("Failed to check current directory.");
                            let directory =
                                pwd.file_name().expect("Failed to get current directory.");
                            Some(directory.to_string_lossy().to_string())
                        })
                        .unwrap();
                    let project_path =
                        std::env::current_dir().expect("Failed to read current directory");
                    let project_name = ProjectName::parse(new_project_name);
                    let project = project::Project::new(project_name.clone(), Some(project_path));
                    project.save();

                    let mut database = Database::from_file(database_path.unwrap())
                        .expect("Failed to read krabby database.");
                    database
                        .add_project(
                            project_name,
                            std::env::current_dir().expect("Failed to get current directory"),
                        )
                        .expect("Failed to add project file.");
                    database.save();
                    exit(0);
                }
                ProjectCommands::Cd { project_name } => {
                    let project_name = project::ProjectName::parse(project_name);
                    let database = Database::from_file(database_path.unwrap())
                        .expect("Failed to read krabby database.");
                    match database.go_to_project(&project_name) {
                        Ok(_) => {
                            println!("echo \"You're good to go!\"");
                            exit(0);
                        }
                        Err(e) => {
                            println!("{}", e);
                            exit(1);
                        }
                    }
                }
            }
        }
        Commands::Hook(hook) => {
            let hook_cmd = hook.command.as_ref().unwrap();

            match hook_cmd {
                HookCommands::Set { hook } => {
                    let hook = match hook.len() {
                        1 => Some(ProjectHook::Simple(
                            script::Command::parse(hook[0].to_string()).to_string(),
                        )),
                        0 => None,
                        _ => Some(ProjectHook::ScriptArray(
                            hook.iter()
                                .map(|script| script::ScriptName::parse(script.to_string()))
                                .collect::<Vec<script::ScriptName>>(),
                        )),
                    };

                    let mut project = project::Project::from_file(project_file_path.unwrap())
                        .expect("Failed to read project_file");
                    match project.set_hook(hook) {
                        Ok(Some(cmd)) => {
                            project.save();
                            println!(
                                "The hook was set {}!\n{} {}",
                                "successfully".green().bold(),
                                "[Hook]:".green().bold(),
                                cmd.bold()
                            );
                            exit(0);
                        }
                        Ok(None) => {
                            project.save();
                            // println!("There was {}.", "no hook found".red().bold());
                            println!("The hook was {}", "removed.".red().bold());
                            exit(0);
                        }
                        Err(e) => {
                            println!("Failed to set hook.\n{}", e);
                            exit(1)
                        }
                    }
                }
            }
        }
        Commands::Script(script) => {
            let script_cmd = script.command.as_ref().unwrap();
            match script_cmd {
                ScriptCommands::Remove { script } => {
                    let script_name = script::ScriptName::parse(script.into());
                    let mut project = project::Project::from_file(project_file_path.unwrap())
                        .expect("Failed to read project file.");
                    match project.remove_script(script_name.clone()) {
                        Ok(_) => {
                            println!("Script {} was removed.", script_name.bold());
                            project.save();
                            exit(0);
                        }
                        Err(e) => {
                            println!("Failed to remove script {}.\n{}", script_name.bold(), e);
                            exit(1);
                        }
                    }
                }
                ScriptCommands::List => {
                    let project = project::Project::from_file(project_file_path.unwrap())
                        .expect("Failed to open project file.");
                    if project.scripts.is_empty() {
                        println!(
                            "Oops! It looks like you have no script registered yet.\n{}",
                            Message::RegisterScript,
                        );
                        exit(0);
                    }
                    println!("So, let's see what do we got here!");
                    for (name, cmd) in project.scripts {
                        println!("\t- {}: {}", name.bold(), cmd.bold());
                    }
                    println!("And that's it!");
                    exit(0);
                }
                ScriptCommands::Add {
                    script_name,
                    script_command,
                } => {
                    let mut project =
                        project::Project::from_file(project_file_path.unwrap()).unwrap();
                    let name = script::ScriptName::parse(script_name.to_string());
                    let command = script::Command::parse(script_command.to_string());
                    let script = krabby::script::Script::new(name.clone(), command);
                    match project.add_script(name.clone(), script) {
                        Ok(_) => {
                            println!("The script {} was registered successfully!", name.bold());
                            project.save();
                            exit(0);
                        }
                        Err(e) => {
                            println!("Failed to add project.\n{}", e);
                            exit(1);
                        }
                    }
                }
            }
        }
        Commands::Run { script } => {
            // Detect `krabby.toml` file and find script. Echoes the command to be evaluated by shell
            // TODO: Improve error report when no file is found/detected.
            let project = project::Project::from_file(project_file_path.unwrap())
                .expect("Failed to open project file");
            let script_name = script::ScriptName::parse(script);
            let script = project.scripts.get(&script_name).expect("Script not found");
            script.echo();
            exit(0);
        }
        Commands::Shell(shell) => {
            let shell_cmd = shell.command.as_ref().unwrap();

            match shell_cmd {
                ShellCommands::Bash | ShellCommands::Zsh => {
                    let script = krabby::shell::Shell(Flavor::Bash).script();
                    print!("{}", script);
                    exit(1);
                }
            }
        }
    }
}

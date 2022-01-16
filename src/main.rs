pub mod database;
pub mod project_info;
pub mod script;

use crate::database::Database;
use crate::script::Script;

use clap::{App, AppSettings, Arg};
use std::{env, process::exit};

fn main() {
    let home = dirs::home_dir().unwrap();
    let default_cfg_file_path = home.join(".krabby.toml");
    let default_cfg_file_path_as_str = default_cfg_file_path.to_str().unwrap();

    let matches = App::new("krabby")
        .version("1.0")
        .setting(AppSettings::ArgRequiredElseHelp)
        .arg(
            Arg::new("project")
                .takes_value(true)
                .value_name("PROJECT")
                .index(1),
        )
        .arg(
            Arg::new("config")
                .short('c')
                .long("config")
                .takes_value(true)
                .value_name("CONFIG")
                .global(true)
                .default_value(default_cfg_file_path_as_str),
        )
        .arg(Arg::new("debug").short('d').takes_value(false).global(true))
        .subcommand(
            App::new("cd").about("change to project path").arg(
                Arg::new("project")
                    .takes_value(true)
                    .value_name("PROJECT")
                    .index(1)
                    .required(true),
            ),
        )
        .subcommand(
            App::new("run")
                .about("run the given script on the project")
                .arg(
                    Arg::new("script")
                        .takes_value(true)
                        .required(true)
                        .help("script to be run")
                        .value_name("SCRIPT"),
                ),
        )
        .subcommand(
            App::new("add")
                .about("adds a new project or script to the configuration")
                .subcommands(vec![
                    App::new("project")
                        .arg(
                            Arg::new("project_name")
                                .takes_value(true)
                                .value_name("NEW_PROJECT")
                                .required(true), // .multiple_values(true), // .index(1),
                        )
                        .arg(
                            Arg::new("project_path")
                                .takes_value(true)
                                .value_name("PROJECT_PATH")
                                .required(true),
                        ),
                    App::new("script")
                        .arg(
                            Arg::new("alias")
                                .takes_value(true)
                                .value_name("ALIAS")
                                .required(true),
                        )
                        .arg(
                            Arg::new("command")
                                .takes_value(true)
                                .value_name("COMMAND")
                                .required(true),
                        ),
                ]),
        )
        .get_matches();

    let db_file_path = matches.value_of("config").unwrap();
    let mut db = match Database::from_file_path(db_file_path) {
        Ok(db) => db,
        Err(_) => exit(1),
    };
    if matches.is_present("debug") {
        println!("Running on debug mode.");
        println!("\n{}", db);
    }

    if matches.is_present("project") {
        let project_index = matches.index_of("project").unwrap();
        if project_index == 1 {
            let project_name = matches.value_of("project").unwrap();
            if db.change_to_project_path(project_name).is_ok() {
                exit(0)
            }
        }
    }

    match matches.subcommand() {
        Some(("run", subcmd)) => {
            let script_alias = subcmd.value_of("script").unwrap();
            match db.detect_project_from_path(env::current_dir().unwrap()) {
                Some(project) => match project.get_script(script_alias) {
                    Ok(script) => script.run(),
                    Err(_) => {
                        eprintln!("soo... there's no script for this project with that name");
                    }
                },
                None => {
                    eprintln!(
                        "i dont know how to say this but you're not in a project directory.\nyou can change this with `kb cd project_name``"
                    );
                    exit(1)
                }
            }
        }
        Some(("cd", subcmd)) => {
            let project_name = subcmd.value_of("project").unwrap();
            if let Some(project) = db.get_project(project_name) {
                match project.change_to_project_path() {
                    Ok(output_string) => {
                        println!("{}", output_string);
                        exit(0)
                    }
                    Err(_) => {
                        eprintln!("oops cant go there");
                        exit(1)
                    }
                };
            } else {
                eprintln!("project {} was not found.", project_name);
                exit(1)
            }
        }
        Some(("add", subcmd)) => {
            match subcmd.subcommand() {
                Some(("project", subcmd)) => {
                    let project_name = subcmd.value_of("project_name").unwrap();
                    let project_path = subcmd.value_of("project_path").unwrap();
                    db.add_project_from_name_and_path(project_name, project_path)
                        .unwrap_or_else(|_| {
                            eprintln!("failed to add project");
                            exit(1)
                        });
                    db.sync().expect(
                        "i hope you memorized the database, because i couldnt save the database",
                    );
                }
                Some(("script", subcmd)) => {
                    println!("Adding script");
                    let _project = {
                        if matches.is_present("project") {
                            db.get_project(subcmd.value_of("project").unwrap())
                                .expect("Failed to find project")
                        } else if let Some(p) =
                            db.detect_project_from_path(env::current_dir().unwrap())
                        {
                            p
                        } else {
                            exit(1)
                        }
                    };
                    let alias = subcmd.value_of("alias").unwrap();
                    let command = subcmd.value_of("command").unwrap();
                    println!("{}", Script::new(alias, command));
                    let _script = Script::new(alias, command);
                    // project.add_script(script);
                }
                Some((_, _)) => {
                    unreachable!()
                }
                None => println!("oi"),
            }
        }
        _ => {
            eprintln!("sheesh. dont know what to do");
            exit(1);
        }
    }
}

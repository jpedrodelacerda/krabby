use clap::{App, AppSettings, Arg};

use serde_derive::{Deserialize, Serialize};
use std::env;
use std::ffi::OsString;
use std::fmt;
use std::fs::File;
use std::io::prelude::*;
use std::io::{Error, ErrorKind};
use std::path::{Path, PathBuf};
use std::process::{exit, Command};

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

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct Database {
    author: String,
    projects: Option<Vec<ProjectInfo>>,
    #[serde(skip)]
    file: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct ProjectInfo {
    name: String,
    path: String,
    scripts: Option<Vec<Script>>,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct Script {
    alias: String,
    command: String,
}

impl Database {
    fn add_project(&mut self, project: ProjectInfo) -> Result<&mut Self, Error> {
        let remember_project_name = project.name.clone();
        if !Path::new(&project.path).is_dir() {
            return Err(Error::new(
                ErrorKind::NotFound,
                "this path leads to nowhere",
            ));
        };
        match &mut self.projects {
            Some(project_list) => {
                project_list.push(project);
            }
            None => {
                self.projects = Some(vec![project]);
            }
        }
        println!("Project {} was created.", remember_project_name);
        Ok(self)
    }

    fn add_project_from_name_and_path(
        &mut self,
        project_name: &str,
        project_path: &str,
    ) -> Result<&mut Self, Error> {
        let project = ProjectInfo::new(project_name, project_path).unwrap();
        if let Some(project_list) = &mut self.projects {
            for p in project_list {
                if p.name == project_name {
                    return Err(Error::new(
                        ErrorKind::AlreadyExists,
                        "a project with this name already exists",
                    ));
                }
                if p.path == project_path {
                    return Err(Error::new(
                        ErrorKind::AlreadyExists,
                        "a project on this path already exists",
                    ));
                }
            }
            self.add_project(project).expect("failed to add project");
        }
        Ok(self)
    }

    fn change_to_project_path(&self, project_name: &str) -> Result<(), Error> {
        match self.get_project(project_name) {
            Some(project) => {
                if Path::new(project.path.as_str()).is_dir() {
                    println!("{}", project.change_to_project_path().unwrap());
                    Ok(())
                } else {
                    eprintln!("project {} path is invalid.", project_name);
                    Err(Error::new(ErrorKind::Other, "project path is invalid"))
                }
            }
            None => {
                eprintln!("Project {} was not found.", project_name);
                Err(Error::new(ErrorKind::NotFound, "project not found"))
            }
        }
    }

    fn detect_project_from_path(&self, _path: PathBuf) -> Option<&ProjectInfo> {
        if let Some(projects) = &self.projects {
            for project in projects {
                if project.pwd_is_project_path() {
                    return Some(project);
                }
            }
        } else {
            return None;
        }
        None
    }

    fn get_project(&self, project_name: &str) -> Option<&ProjectInfo> {
        if let Some(projects) = &self.projects {
            for project in projects {
                if project.name == project_name {
                    return Some(project);
                }
            }
        } else {
            return None;
        }
        None
    }

    fn from_file_path(file_path: &str) -> Result<Database, Error> {
        let mut f = match File::open(file_path) {
            Ok(contents) => contents,
            Err(e) => {
                match e.kind() {
                    ErrorKind::NotFound => {
                        eprintln!("Could not find config file at {}.", file_path)
                    }
                    _ => eprintln!("maybe i should know how to handle this. but i dont."),
                }
                return Err(e);
            }
        };

        let mut db_str = String::new();
        match f.read_to_string(&mut db_str) {
            Ok(_) => {
                let db = Database::new(db_str.as_str(), Some(file_path));
                Ok(db)
            }
            Err(e) => {
                match e.kind() {
                    ErrorKind::NotFound => eprintln!("missing file"),
                    _ => eprintln!("maybe i should know how to handle this. but i dont."),
                }
                Err(e)
            }
        }
    }

    fn new(raw_data: &str, path: Option<&str>) -> Self {
        let mut db: Database = toml::from_str(raw_data).unwrap();
        if let Some(p) = path {
            db.file = Some(p.to_string())
        }

        db
    }

    fn empty() -> Self {
        Database {
            author: "John Doe <john@doe.com>".to_string(),
            projects: None,
            file: None,
        }
    }

    fn list_projects(&self) -> Vec<&str> {
        let mut proj_vec = vec![];
        let project_list = self.projects.as_ref().unwrap();
        project_list.iter().for_each(|project_name| {
            proj_vec.push(project_name.name.as_str());
        });
        proj_vec
    }

    fn sync(&self) -> Result<&Self, Error> {
        let path = &self.file;

        let mut f = File::create(path.as_ref().unwrap())?;
        let s = toml::to_string(&self).unwrap();
        if let Err(e) = f.write_all(s.as_bytes()) {
            return Err(e);
        }
        Ok(self)
    }
}

impl ProjectInfo {
    fn new(project: &str, path: &str) -> Result<Self, Error> {
        Ok(ProjectInfo {
            name: project.to_string(),
            path: path.to_string(),
            scripts: None,
        })
    }

    fn pwd_is_project_path(&self) -> bool {
        let pwd = env::current_dir().unwrap().into_os_string();
        let path_os_str: OsString = self.path.clone().into();
        if pwd.eq(&path_os_str) {
            return true;
        }

        false
    }

    fn change_to_project_path(&self) -> Result<String, Error> {
        if !Path::new(&self.path).is_dir() {
            return Err(Error::new(
                ErrorKind::NotFound,
                "this path leads to nowhere",
            ));
        };
        Ok(format!("cd {}", self.get_path().unwrap()))
    }

    fn get_path(&self) -> Result<&str, Error> {
        Ok(&self.path)
    }

    fn get_script(&self, script_alias: &str) -> Result<&Script, Error> {
        match &self.scripts {
            Some(scripts) => {
                for script in scripts {
                    if script.alias == script_alias {
                        return Ok(script);
                    }
                }
                Err(Error::new(ErrorKind::NotFound, "no script found"))
            }
            None => Err(Error::new(
                ErrorKind::NotFound,
                "no scripts configured for this project",
            )),
        }
    }

    fn add_script(&mut self, script: Script) -> Result<&Self, Error> {
        match &mut self.scripts {
            Some(script_list) => {
                // for s in script_list {
                //     if s.alias == script.alias {
                //         return Err(Error::new(
                //             ErrorKind::AlreadyExists,
                //             "script already exists",
                //         ));
                //     }
                // }
                script_list.push(script);
            }
            None => {
                self.scripts = Some(vec![script]);
            }
        }
        Ok(self)
        // match &self.scripts {
        //     Some(scripts) => {
        //         for s in scripts {
        //             if script.alias == s.alias {
        //                 return Err(Error::new(ErrorKind::AlreadyExists, "no script found"));
        //             }
        //         }
        //         scripts.push(script);
        //         return Ok(self);
        //     }
        //     None => {
        //         // Some scripts =
        //     }
        // }
    }
}

impl fmt::Display for Database {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "Author: {}", self.author)?;
        if let Some(projects) = &self.projects {
            projects.iter().for_each(|project| {
                writeln!(f, "{}", project).expect("failed to display project");
            });
        }
        Ok(())
    }
}

impl fmt::Display for ProjectInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "[Project] => {}", self.name)?;
        writeln!(f, "[Path] => {}", self.path)?;
        match &self.scripts {
            Some(scripts) => {
                writeln!(f, "[Scripts] =>")?;
                for script in scripts {
                    writeln!(f, "\t{}", script)?;
                }
            }
            None => {
                write!(f, "no script configured")?;
            }
        }
        Ok(())
    }
}

impl Script {
    fn new(alias: &str, command: &str) -> Self {
        Script {
            alias: alias.to_string(),
            command: command.to_string(),
        }
    }

    fn format(&self) -> String {
        self.command.to_string()
    }

    fn run(&self) {
        // println!("{}", self.command)
        Command::new("sh")
            .arg("-c")
            .arg(self.format())
            .spawn()
            .expect("failed to run script");
    }
}

impl fmt::Display for Script {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "\"{}\" -> \"{}\"", self.alias, self.command)
        // Ok(())
    }
}

#[test]
fn empty_database() {
    let raw_db = r#"author = "John Doe <john@doe.com>"
"#;

    let empty_db = Database::empty();
    let deserialized_db = toml::to_string(&empty_db).unwrap();
    assert_eq!(raw_db, deserialized_db);
}

#[test]
fn add_first_project() {
    let raw_config = r#"author = "John Doe <john@doe.com>"

[[projects]]
name = "krabby"
path = "/tmp/"
"#;

    let krabby_project = ProjectInfo {
        name: String::from("krabby"),
        path: String::from("/tmp/"),
        scripts: None,
    };

    // let db = Database {
    //     author: String::from("John Doe <john@doe.com>"),
    //     projects: Some(vec![krabby_project]),
    //     file: None,
    // };
    let mut db = Database::empty();
    db.add_project(krabby_project).unwrap();
    let db_serialized = toml::to_string(&db).unwrap();
    println!("but looks like this:\n{}", db_serialized);

    assert_eq!(raw_config, db_serialized)
}

#[test]
fn missing_database() {
    let missing_file_path = std::path::Path::new("/i/shouldnt/exist");
    if let Err(e) = Database::from_file_path(missing_file_path.to_str().unwrap()) {
        assert_eq!(e.kind(), ErrorKind::NotFound);
    } else {
        panic!("i should not be here");
    }
}

#[test]
fn cd_to_invalid_path_must_fail() {
    let _db = Database::empty();

    let project = ProjectInfo::new("nopath", "/but/through").unwrap();
    if let Err(e) = project.change_to_project_path() {
        return assert_eq!(e.kind(), ErrorKind::NotFound);
    }
    panic!("i should not /* be */ /* here */")
}

#[test]
fn add_project_with_invalid_path_must_fail() {
    let mut db = Database::empty();

    if let Err(e) = db.add_project_from_name_and_path("nopath", "/but/through/") {
        return assert_eq!(e.kind(), ErrorKind::NotFound);
    }
}

#[test]
fn add_project_with_valid_path_must_pass() {
    let mut db = Database::empty();
    db.add_project_from_name_and_path("not_spiderman", "/tmp/")
        .unwrap();
}

#[test]
fn test_serialization() {
    let raw_config = r#"author = "John Doe <john@doe.com>"

[[projects]]
name = "krabby"
path = "/tmp/krabby"

[[projects.scripts]]
alias = "hello"
command = "echo hello"

[[projects.scripts]]
alias = "world"
command = "echo world"

[[projects]]
name = "tmp"
path = "/tmp"

[[projects.scripts]]
alias = "world"
command = "echo world"
"#;

    println!("should look like:\n{}", raw_config);
    let krabby_scripts: Vec<Script> = vec![
        Script::new("hello", "echo hello"),
        Script::new("world", "echo world"),
    ];
    let krabby_project = ProjectInfo {
        name: String::from("krabby"),
        path: String::from("/tmp/krabby"),
        scripts: Some(krabby_scripts),
    };
    let tmp_scripts = vec![Script::new("world", "echo world")];
    let tmp_project = ProjectInfo {
        name: String::from("tmp"),
        path: String::from("/tmp"),
        scripts: Some(tmp_scripts),
    };

    let db = Database {
        author: String::from("John Doe <john@doe.com>"),
        projects: Some(vec![krabby_project, tmp_project]),
        file: None,
    };
    let db_serialized = toml::to_string(&db).unwrap();
    println!("but looks like this:\n{}", db_serialized);

    assert_eq!(raw_config, db_serialized)
}

#[test]
fn test_deserialization() {
    let raw_config = r#"author = "John Doe <john@doe.com>"

[[projects]]
name = "krabby"
path = "/tmp/krabby"

[[projects.scripts]]
alias = "hello"
command = "echo hello"

[[projects.scripts]]
alias = "world"
command = "echo world"

[[projects]]
name = "tmp"
path = "/tmp"

[[projects.scripts]]
alias = "world"
command = "echo world"
"#;

    let db = Database::new(raw_config, None);

    let krabby_scripts = vec![
        Script::new("hello", "echo hello"),
        Script::new("world", "echo world"),
    ];
    let krabby_project = ProjectInfo {
        name: String::from("krabby"),
        path: String::from("/tmp/krabby"),
        scripts: Some(krabby_scripts),
    };
    let tmp_scripts = vec![Script::new("world", "echo world")];
    let tmp_project = ProjectInfo {
        name: String::from("tmp"),
        path: String::from("/tmp"),
        scripts: Some(tmp_scripts),
    };

    assert_eq!(
        db,
        Database {
            author: String::from("John Doe <john@doe.com>"),
            projects: Some(vec![krabby_project, tmp_project],),
            file: None,
        }
    )
}

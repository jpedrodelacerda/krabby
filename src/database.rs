use crate::project_info::ProjectInfo;
use serde_derive::{Deserialize, Serialize};
use std::fmt;
use std::fs::File;
use std::io::prelude::*;
use std::io::{Error, ErrorKind};
use std::path::{Path, PathBuf};

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct Database {
    pub author: String,
    pub projects: Option<Vec<ProjectInfo>>,
    #[serde(skip)]
    file: Option<String>,
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

    pub fn add_project_from_name_and_path(
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

    pub fn change_to_project_path(&self, project_name: &str) -> Result<(), Error> {
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

    pub fn detect_project_from_path(&self, _path: PathBuf) -> Option<&ProjectInfo> {
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

    pub fn get_project(&self, project_name: &str) -> Option<&ProjectInfo> {
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

    pub fn from_file_path(file_path: &str) -> Result<Database, Error> {
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

    pub fn new(raw_data: &str, path: Option<&str>) -> Self {
        let mut db: Database = toml::from_str(raw_data).unwrap();
        if let Some(p) = path {
            db.file = Some(p.to_string())
        }

        db
    }

    pub fn empty() -> Self {
        Database {
            author: "John Doe <john@doe.com>".to_string(),
            projects: None,
            file: None,
        }
    }

    pub fn list_projects(&self) -> Vec<&str> {
        let mut proj_vec = vec![];
        let project_list = self.projects.as_ref().unwrap();
        project_list.iter().for_each(|project_name| {
            proj_vec.push(project_name.name.as_str());
        });
        proj_vec
    }

    pub fn sync(&self) -> Result<&Self, Error> {
        let path = &self.file;

        let mut f = File::create(path.as_ref().unwrap())?;
        let s = toml::to_string(&self).unwrap();
        if let Err(e) = f.write_all(s.as_bytes()) {
            return Err(e);
        }
        Ok(self)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::script::Script;

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
}

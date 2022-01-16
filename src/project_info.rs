use std::path::Path;
use std::{env, fmt};

use serde_derive::{Deserialize, Serialize};
use std::ffi::OsString;
use std::io::{Error, ErrorKind};

use crate::script::Script;

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct ProjectInfo {
    pub name: String,
    pub path: String,
    pub scripts: Option<Vec<Script>>,
}

impl ProjectInfo {
    pub fn new(project: &str, path: &str) -> Result<Self, Error> {
        Ok(ProjectInfo {
            name: project.to_string(),
            path: path.to_string(),
            scripts: None,
        })
    }

    pub fn pwd_is_project_path(&self) -> bool {
        let pwd = env::current_dir().unwrap().into_os_string();
        let path_os_str: OsString = self.path.clone().into();
        if pwd.eq(&path_os_str) {
            return true;
        }

        false
    }

    pub fn change_to_project_path(&self) -> Result<String, Error> {
        if !Path::new(&self.path).is_dir() {
            return Err(Error::new(
                ErrorKind::NotFound,
                "this path leads to nowhere",
            ));
        };
        Ok(format!("cd {}", self.get_path().unwrap()))
    }

    pub fn get_path(&self) -> Result<&str, Error> {
        Ok(&self.path)
    }

    pub fn get_script(&self, script_alias: &str) -> Result<&Script, Error> {
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

    pub fn add_script(&mut self, script: Script) -> Result<&Self, Error> {
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn cd_to_invalid_path_must_fail() {
        let project = ProjectInfo::new("nopath", "/but/through").unwrap();
        if let Err(e) = project.change_to_project_path() {
            return assert_eq!(e.kind(), ErrorKind::NotFound);
        }
        panic!("i should not /* be */ /* here */")
    }
}

use std::fmt::{self, Display};

use owo_colors::OwoColorize;

use crate::project::ProjectName;

pub enum Message {
    RegisterProject,
    RegisterScript,
    ProjectNotFound(ProjectName),
    ProjectFileNotFound(ProjectName, String),
    RegisterProjectSuccess(ProjectName),
    RegisterProjectFail(ProjectName, anyhow::Error),
}

impl Message {
    pub fn println(self) {
        println!("{}", self)
    }
}

impl Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let message = match self {
            Self::RegisterProject => {
                format!(
                    "You can {} a new project with {}",
                    "register".green().bold(),
                    "'kb project add PROJECT_NAME PROJECT_PATH'".bold()
                )
            }
            Self::RegisterScript => {
                format!(
                    "You can {} a new script with {}",
                    "register".green().bold(),
                    "'kb script add SCRIPT_NAME SCRIPT_COMMAND'".bold()
                )
            }
            Self::ProjectNotFound(project_name) => {
                format!(
                    "Project {} was {} on database! Are you sure you registered it?",
                    project_name.bold(),
                    "not found".red().bold(),
                )
            }
            Self::ProjectFileNotFound(project_name, path) => {
                format!("I dont know how to tell you this, but there was a problem with the registry.\nApparently the {} directory {} or {} {}.", project_name.bold(), "is missing".red().bold(), path, "is not a directory".red().bold())
            }
            Self::RegisterProjectSuccess(project_name) => {
                format!("{} was registered successfully!", project_name.bold())
            }

            Self::RegisterProjectFail(project_name, error) => {
                format!(
                    "There was a problem registering {}!\n{}",
                    project_name.bold(),
                    error
                )
            }
        };
        write!(f, "{}", message)
    }
}

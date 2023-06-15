use crate::{
    messages::Message,
    project::{Project, ProjectName},
};
use anyhow::{anyhow, Error};
use indexmap::IndexMap;
use owo_colors::OwoColorize;
use serde::{Deserialize, Serialize};
use std::{
    fs::OpenOptions,
    io::{Read, Write},
    path::PathBuf,
    str::FromStr,
};

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Database {
    pub projects: IndexMap<ProjectName, PathBuf>,
    #[serde(skip)]
    pub path: Option<PathBuf>,
}

impl Database {
    pub fn new(path: Option<PathBuf>) -> Self {
        Self {
            projects: IndexMap::new(),
            path,
        }
    }

    pub fn list_projects(&self) -> Vec<ProjectName> {
        self.projects
            .iter()
            .map(|(project_name, _project_path)| project_name.clone())
            .collect()
    }

    pub fn remove_project(&mut self, project_name: ProjectName) -> Result<(), Error> {
        match self.projects.shift_remove(&project_name) {
            Some(_p) => Ok(()),
            None => Err(anyhow!(
                "There was no project named {} on the database!",
                project_name.bold()
            )),
        }
    }

    pub fn add_project(&mut self, project_name: ProjectName, path: PathBuf) -> Result<(), Error> {
        // If we don't canonicalize, the path will be useless when we try to `cd` into it.
        let path = std::fs::canonicalize(path).expect("Failed to canonicalize path");
        if let Some(p) = self.projects.get(&project_name) {
            return Err(anyhow!(
                "{} is registered already at {}.",
                project_name,
                p.to_str().unwrap().bold()
            ));
        }
        for (project_name, project_path) in self.projects.iter() {
            if project_path == &path {
                return Err(anyhow!(
                    "The project at {} is already registered under the name {}.",
                    project_path.to_str().unwrap().bold(),
                    project_name.bold()
                ));
            }
        }
        self.projects.insert(project_name, path);
        Ok(())
    }

    pub fn get_project_path(&self, project_name: &ProjectName) -> Option<&PathBuf> {
        self.projects.get(project_name)
    }

    // Return Ok((cd_command, Some(hook_command))) in case of success
    pub fn go_to_project(
        &self,
        project_name: &ProjectName,
    ) -> Result<(String, Option<String>), Error> {
        match self.projects.get(project_name) {
            Some(project_path) => {
                if !project_path.is_dir() {
                    return Err(anyhow!("I dont know how to tell you this, but there was a problem with the registry.\nApparently {} directory {} or {}.", project_path.to_string_lossy().bold(), "is missing".red().bold(), "is not a directory".red().bold()));
                }
                println!("echo \"Krabby is taking you to {}!\";", project_name.bold());
                let s = project_path.to_string_lossy();
                let cd_cmd = format!("cd {}", s);
                println!("{}", cd_cmd);
                // Checks for `krabby.toml` project file to see if there are any hooks to run
                if let Some(hook_cmd) = self.get_project_hook_cmd(project_name) {
                    println!("echo \"Running hook:\n{}\"", hook_cmd.bold());
                    println!("{}", hook_cmd);
                    return Ok((cd_cmd, Some(hook_cmd)));
                }
                Ok((cd_cmd, None))
            }
            None => Err(anyhow!(
                "{}",
                Message::ProjectNotFound(project_name.clone())
            )),
        }
    }

    pub fn get_project_hook_cmd(&self, project_name: &ProjectName) -> Option<String> {
        if let Ok(project) = self.get_project_file(project_name) {
            return project.get_hook_cmd();
        }
        None
    }

    pub fn get_project(&self, project_name: &ProjectName) -> Option<&PathBuf> {
        self.projects.get(project_name)
    }

    fn get_project_file(&self, project_name: &ProjectName) -> Result<Project, anyhow::Error> {
        let project_file_path = match self.get_project(project_name) {
            Some(path) => path.join("krabby.toml"),
            None => return Err(anyhow!("There was no project {}", project_name)),
        };
        Project::from_file(project_file_path)
    }

    pub fn from_string(s: &str) -> Self {
        toml::from_str(s).unwrap()
    }

    fn set_path(&mut self, path: PathBuf) {
        self.path = Some(path);
    }

    pub fn save(&self) {
        self.write().unwrap_or_else(|_| {
            panic!(
                "Failed to save database at {}",
                self.path
                    .clone()
                    .unwrap()
                    .into_os_string()
                    .into_string()
                    .unwrap()
            );
        });
    }

    pub fn write(&self) -> Result<(), Error> {
        let mut f = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            // So we can rewrite the whole file
            .truncate(true)
            .create(true)
            .open(self.path.clone().expect("Path was not set properly"))?;
        f.write_all(self.to_string().as_bytes())?;
        Ok(())
    }

    pub fn from_file(path: PathBuf) -> Result<Self, Error> {
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(path.clone())?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        if contents.is_empty() {
            file.write_all(Database::new(Some(path.clone())).to_string().as_bytes())?;
        }
        let mut db = Self::from_str(&contents)
            .unwrap_or_else(|e| panic!("Failed to read database string.\n{}", e));
        db.set_path(path);
        Ok(db)
    }
}

impl FromStr for Database {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let database: Result<Self, toml::de::Error> = toml::from_str(s);
        if let Ok(db) = database {
            return Ok(db);
        }
        Err(anyhow!("Failed to parse Database from string:\n{}", s))
    }
}

impl ToString for Database {
    fn to_string(&self) -> String {
        toml::to_string(self).unwrap()
    }
}

impl Default for Database {
    fn default() -> Self {
        Self::new(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn database_is_initialized_empty() {
        let database = Database::new(None);
        assert!(database.projects.is_empty())
    }

    #[test]
    fn parse_database_successfully() {
        let mut database = Database::new(None);
        database
            .add_project(ProjectName::parse("project".into()), "/".into())
            .unwrap();

        let database_str = r#"
            [projects]
            project = "/"
        "#;
        let database_from_str: Database = toml::from_str(database_str).unwrap();

        assert_eq!(database, database_from_str)
    }

    #[test]
    fn add_project_to_database_successfully() {
        let mut database = Database::new(None);
        let res = database.add_project(ProjectName::parse("project".into()), "/".into());
        assert!(res.is_ok());
        let project_list: Vec<ProjectName> = vec![ProjectName::parse("project".into())];
        assert_eq!(database.list_projects(), project_list);
    }

    #[test]
    fn remove_existing_project_from_database() {
        let database_str = r#"
            [projects]
            project1 = "/tmp/project1"
            project2 = "/tmp/project2"
        "#;
        let mut database = Database::from_string(database_str);
        database.set_path("./krabby.db".into());
        database.save();
        database
            .remove_project(ProjectName::parse("project2".into()))
            .unwrap();
        database.save();

        let single_project_database_str = r#"[projects]
project1 = "/tmp/project1"
"#;
        assert_eq!(database.to_string(), single_project_database_str);

        database
            .remove_project(ProjectName::parse("project1".into()))
            .unwrap();
        database.save();

        let empty_database_str = r#"[projects]
"#;

        assert_eq!(database.to_string(), empty_database_str);
    }

    #[test]
    #[should_panic]
    fn remove_project_from_empty_database() {
        let mut database = Database::new(None);
        database
            .remove_project(ProjectName::parse("project".into()))
            .expect("failed to delete project")
    }

    use crate::project::{Project, ProjectName};
    use rand::distributions::Alphanumeric;
    use rand::{thread_rng, Rng};
    use std::fs;

    fn create_random_database_file(path: &str) -> PathBuf {
        let mut path = PathBuf::from(path);
        let rand_string: String = thread_rng()
            .sample_iter(&Alphanumeric)
            .take(30)
            .map(char::from)
            .collect();
        let database_file_name = format!("{}-krabby.db", rand_string.clone());
        path.push(database_file_name);

        Database::new(Some(path.clone()));

        path
    }

    fn create_random_project(path: &str) -> (ProjectName, PathBuf) {
        let mut path = PathBuf::from(path);
        let rand_string: String = thread_rng()
            .sample_iter(&Alphanumeric)
            .take(30)
            .map(char::from)
            .collect();
        let project_name = ProjectName::parse(format!("{}-project", rand_string.clone()));
        path.push(project_name.to_string());

        fs::create_dir(path.clone())
            .unwrap_or_else(|_| panic!("Failed to create {}", path.clone().to_string_lossy()));
        Project::new(project_name.clone(), Some(path.clone()));

        (project_name, path)
    }

    fn remove_file(path: &str) {
        fs::remove_file(path).unwrap_or_else(|_| panic!("Failed to remove {}", path))
    }
    fn remove_dir(path: &str) {
        fs::remove_dir(path).unwrap_or_else(|_| panic!("Failed to remove {}", path))
    }
}

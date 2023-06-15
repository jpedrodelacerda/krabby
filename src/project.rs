use crate::{
    hook::ProjectHook,
    script::{Command, Script, ScriptName},
};
use anyhow::{anyhow, Error};
use indexmap::IndexMap;
use owo_colors::OwoColorize;
use serde::{Deserialize, Serialize};
use std::{
    fmt::{self, Display},
    fs::OpenOptions,
    io::{Read, Write},
    path::PathBuf,
    str::FromStr,
};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct Project {
    pub name: ProjectName,
    #[serde(skip)]
    pub path: Option<PathBuf>,
    pub hook: Option<ProjectHook>,
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    pub scripts: IndexMap<ScriptName, Script>,
}

impl Project {
    pub fn new(name: ProjectName, path: Option<PathBuf>) -> Self {
        Project {
            name,
            path,
            scripts: IndexMap::new(),
            hook: None,
        }
    }

    pub fn populate_names(&mut self) -> Result<(), anyhow::Error> {
        for (name, script) in &mut self.scripts {
            script.set_name(name.clone())
        }
        Ok(())
    }

    pub fn scripts(&self) -> &IndexMap<ScriptName, Script> {
        &self.scripts
    }
    pub fn get_script(&self, script_name: &ScriptName) -> Option<&Script> {
        self.scripts.get(script_name)
    }

    pub fn remove_script(&mut self, script_name: ScriptName) -> Result<(), anyhow::Error> {
        match self.scripts.shift_remove(&script_name) {
            Some(_s) => Ok(()),
            None => Err(anyhow!("{} was not found.", script_name.to_string().bold())),
        }
    }

    pub fn add_script(&mut self, name: ScriptName, script: Script) -> Result<(), anyhow::Error> {
        if self.scripts.contains_key(&name) {
            return Err(anyhow!("Script already exists: {}", &name.bold()));
        }
        let _ = &self.scripts.insert(name, script);
        Ok(())
    }

    pub fn hook(&self) -> &Option<ProjectHook> {
        &self.hook
    }

    /// Defines the hook and return the command or commands.
    /// It returns an `Option` so we can return an `Ok` if the hook is set to None.
    pub fn set_hook(&mut self, hook: Option<ProjectHook>) -> Result<Option<String>, Error> {
        match self.validate_hook(&hook) {
            Ok(_) => {
                self.hook = hook;
                match self.get_hook_cmd() {
                    Some(cmd) => Ok(Some(cmd)),
                    None => Ok(None),
                }
            }
            Err(e) => Err(e),
        }
    }

    pub fn get_hook_cmd(&self) -> Option<String> {
        match self.validate_hook(self.hook()) {
            Ok(_) => match self.hook() {
                Some(ProjectHook::Simple(cmd)) => Some(cmd.to_string()),
                Some(ProjectHook::ScriptArray(hooks)) => {
                    let hooks_cmd = hooks
                        .iter()
                        .map(|s| {
                            self.get_script(s)
                                .unwrap_or_else(|| {
                                    panic!("Hook {} does not match any script", s.bold());
                                })
                                .to_string()
                        })
                        .collect::<Vec<String>>()
                        .join("; ");
                    Some(hooks_cmd)
                }
                None => None,
            },
            Err(e) => {
                panic!("Failed to get hook command.\n{}", e);
            }
        }
    }

    pub fn validate_hook(&self, hook: &Option<ProjectHook>) -> Result<(), Error> {
        match hook {
            Some(ProjectHook::ScriptArray(hooks)) => {
                for h in hooks {
                    if !self
                        .scripts
                        .contains_key(&ScriptName::parse(h.to_string().clone()))
                    {
                        return Err(anyhow!("{} is not a valid hook", h.bold()));
                    }
                }
                Ok(())
            }
            Some(ProjectHook::Simple(hook)) => {
                Command::parse(hook.to_string());
                Ok(())
            }
            None => Ok(()),
        }
    }

    // pub fn from_string(s: &str) -> Self {
    //     toml::from_str::<Project>(s).expect("Failed to parse project string")
    // }

    pub fn save(&self) {
        self.write().unwrap_or_else(|_| {
            panic!(
                "Failed to save project file at {}.",
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
            .create(true)
            // So we can rewrite the whole file
            .truncate(true)
            .open(self.path.clone().expect("Path was not set properly"))?;
        f.write_all(self.to_string().as_bytes())?;
        Ok(())
    }

    fn set_path(&mut self, path: PathBuf) {
        self.path = Some(path);
    }

    pub fn from_file(path: PathBuf) -> Result<Self, Error> {
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(path.clone())?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        if contents.is_empty() {
            let project_name = ProjectName::parse(
                std::env::current_dir()
                    .expect("Failed to check current directory.")
                    .file_name()
                    .expect("Failed to get the current directory")
                    .to_str()
                    .expect("Failed to convert directory to str")
                    .to_string(),
            );

            file.write_all(
                Self::new(project_name, Some(path.clone()))
                    .to_string()
                    .as_bytes(),
            )?;
        }
        let mut project = Self::from_str(&contents)
            .unwrap_or_else(|e| panic!("Failed to read project string.\n{}", e));
        project.set_path(path);
        Ok(project)
    }
}

impl FromStr for Project {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let project: Result<Self, toml::de::Error> = toml::from_str(s);
        if let Ok(mut p) = project {
            let _ = p.populate_names();
            // match project.validate_hook() {}
            // p.validate_hook(p.hook())?
            match p.validate_hook(p.hook()) {
                Ok(_) => {
                    return Ok(p);
                }
                Err(e) => {
                    return Err(anyhow!("Krabby failed to validate project hook.\n{}", e));
                }
            }
        }
        Err(anyhow!("Krabby couldn't parse project from string:\n{}", s))
    }
}

impl ToString for Project {
    fn to_string(&self) -> String {
        toml::to_string(self).unwrap()
    }
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq, Hash)]
#[serde(transparent)]
pub struct ProjectName(String);
impl ProjectName {
    pub fn parse(s: String) -> Self {
        let is_empty = s.trim().is_empty();

        let is_too_long = s.len() > 20;

        let forbidden_characters = ['/', '(', ')', '"', '\'', '<', '>', '\\', '{', '}'];
        let contains_forbidden_characters = s.chars().any(|g| forbidden_characters.contains(&g));

        if is_empty || is_too_long || contains_forbidden_characters {
            panic!("{} is not a valid script name", s);
        }
        Self(s)
    }
}

impl Display for ProjectName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod test {
    use std::panic;

    use crate::script::Command;

    use super::*;

    #[test]
    fn project_initializes_with_no_script() {
        let project = Project::new(ProjectName::parse("validname".into()), None);
        assert!(project.scripts.is_empty());
        assert!(project.hook().is_none());
    }

    #[test]
    fn empty_project_file_parsed_successfully() {
        let project = Project::new(ProjectName::parse("project".into()), None);
        let project_from_str: Project = toml::from_str(
            r#"
            name = "project"
            "#,
        )
        .unwrap();
        assert_eq!(project, project_from_str)
    }

    #[test]
    fn project_simple_hook_file_parsed_successfully() {
        let mut project = Project::new(ProjectName::parse("project".into()), None);
        project
            .set_hook(Some(ProjectHook::Simple(
                "echo \"Welcome to Krabby!\"".to_string(),
            )))
            .unwrap();
        let project_from_str: Project = toml::from_str(
            r#"
            name = "project"
            hook = "echo \"Welcome to Krabby!\""
            "#,
        )
        .unwrap();
        assert_eq!(project, project_from_str)
    }

    #[test]
    fn project_list_hook_file_parsed_successfully() {
        let mut project = Project::new(ProjectName::parse("project".into()), None);
        let script_name = ScriptName::parse("hello".into());
        let script = Script::new(script_name.clone(), Command::parse("echo hello".into()));
        project.add_script(script_name, script).unwrap();
        project
            .set_hook(Some(ProjectHook::ScriptArray(vec![ScriptName::parse(
                "hello".into(),
            )])))
            .unwrap();
        let project_from_str: Project = toml::from_str(
            r#"
            name = "project"
            hook = [ "hello" ]

            [scripts]
            hello = "echo hello"
            "#,
        )
        .unwrap();
        dbg!(&project_from_str);
        assert_eq!(project, project_from_str)
    }

    #[test]
    #[should_panic]
    fn project_fails_to_add_invalid_hook_successfully() {
        let mut project = Project::new(ProjectName::parse("project".into()), None);
        let script_name = ScriptName::parse("hello".into());
        project
            .set_hook(Some(ProjectHook::ScriptArray(vec![script_name])))
            .unwrap();
    }

    #[test]
    #[should_panic]
    fn project_fails_to_parse_invalid_hook() {
        Project::from_str(
            r#"
            name = "project"
            hook = [ "hello" ]
            "#,
        )
        .unwrap();
    }

    #[test]
    fn project_file_with_scripts_is_parsed_successfully() {
        let mut project = Project::new(ProjectName::parse("project".into()), None);

        let name = ScriptName::parse("run".into());
        let script = Script::new(
            name.clone(),
            Command::parse(r#"echo 'Krabby says hi'"#.into()),
        );
        project.add_script(name.clone(), script).unwrap();
        let project_from_str: Project = Project::from_str(
            r#"
            name = "project"

            [scripts]
            run = "echo 'Krabby says hi'"
            "#,
        )
        .unwrap();
        assert_eq!(project, project_from_str)
    }

    #[test]
    fn remove_existing_script_from_project() {
        let database_str = r#"
name = "krabby"

[scripts]
hello = "echo \"hello\""
world = "echo \"world\""
        "#;
        let (_, project_path) = create_random_project_file_from_str(database_str);
        let mut project = Project::from_file(project_path.clone()).unwrap();
        project.save();
        project
            .remove_script(ScriptName::parse("hello".into()))
            .unwrap();
        project.save();

        let single_script_project_str = r#"name = "krabby"

[scripts]
world = "echo \"world\""
"#;
        assert_eq!(project.to_string(), single_script_project_str);

        project
            .remove_script(ScriptName::parse("world".into()))
            .unwrap();
        project.save();

        let empty_project_str = r#"name = "krabby"
"#;

        assert_eq!(project.to_string(), empty_project_str);
        remove_file(&project_path);
    }

    #[test]
    #[should_panic]
    fn project_name_cannot_be_empty() {
        ProjectName::parse("".into());
    }

    #[test]
    #[should_panic]
    fn project_name_cannot_be_blank() {
        ProjectName::parse("  ".into());
    }

    #[test]
    #[should_panic]
    fn project_name_cannot_be_over_20_characters() {
        ProjectName::parse("iamlongerthan20characters".into());
    }

    #[test]
    fn project_name_cannot_contain_forbidden_characters() {
        let cases = [
            ("i/am/not/allowed".to_string(), "cannot contain slashes"),
            ("i(cantbeallowed".to_string(), "cannot contain parenthesis"),
            ("neither)shouldi".to_string(), "cannot contain parenthesis"),
            ("i\\shouldpanic".to_string(), "cannot contain backslashes"),
            (
                "why\"notallowed".to_string(),
                "cannot contain double quotes",
            ),
            ("shouldnot'be".to_string(), "cannot contain single quotes"),
            ("<antthisbegood".to_string(), "cannot contain lt sign"),
            ("cantthisbegoo>".to_string(), "cannot contain gt sign"),
            ("cantcauseof{".to_string(), "cannot contain bracket"),
            ("cantcauseof}".to_string(), "cannot contain bracket"),
        ];
        for (case, msg) in cases {
            let result = panic::catch_unwind(|| ProjectName::parse(case.clone()));
            assert!(
                result.is_err(),
                "{} should be a project script name: {}",
                case,
                msg,
            );
        }
    }

    use rand::distributions::Alphanumeric;
    use rand::{thread_rng, Rng};
    use std::fs;

    fn create_random_project_file() -> (ProjectName, PathBuf) {
        let mut path = std::env::temp_dir();
        let rand_string: String = thread_rng()
            .sample_iter(&Alphanumeric)
            .take(30)
            .map(char::from)
            .collect();
        let project_name = ProjectName::parse(rand_string.clone());
        let project_file_name = format!("{}-krabby.toml", rand_string.clone());
        path.push(project_file_name);

        Project::new(project_name.clone(), Some(path.clone()));

        (project_name, path)
    }

    fn create_random_project_file_from_str(contents: &str) -> (ProjectName, PathBuf) {
        let mut path = std::env::temp_dir();
        let rand_string: String = thread_rng()
            .sample_iter(&Alphanumeric)
            .take(10)
            .map(char::from)
            .collect();
        let project_name = ProjectName::parse(rand_string.clone());
        let project_file_name = format!("{}-krabby.toml", project_name.clone());
        path.push(project_file_name);

        let mut project = Project::from_str(contents).unwrap();
        project.set_path(path.clone());
        project.save();

        (project_name, path)
    }

    fn remove_file(path: &PathBuf) {
        fs::remove_file(path)
            .unwrap_or_else(|_| panic!("Failed to remove {}", path.to_string_lossy()))
    }
}

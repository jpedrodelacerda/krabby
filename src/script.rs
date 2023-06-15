use serde::{Deserialize, Serialize};
use std::fmt::{self, Display};

#[derive(Default, Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
#[serde(transparent)]
pub struct Script {
    pub command: Command,
    #[serde(skip)]
    pub name: ScriptName,
}

impl Script {
    pub fn new(name: ScriptName, command: Command) -> Self {
        Self { name, command }
    }
    pub fn echo(&self) -> String {
        println!("{}", self.command);
        format!("{}", self.command)
    }
    pub fn set_name(&mut self, name: ScriptName) {
        self.name = name;
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash, Clone)]
#[serde(transparent)]
pub struct ScriptName(String);
impl ScriptName {
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
impl Display for ScriptName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Display for Script {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.command)
    }
}

#[derive(Default, Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct Command(String);
impl Command {
    pub fn parse(s: String) -> Self {
        let is_empty = s.trim().is_empty();

        if is_empty {
            return Self(r#"echo 'Krabby says hi!'"#.into());
        }
        Self(s)
    }
}

impl Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Default for ScriptName {
    fn default() -> Self {
        Self::parse("hello".into())
    }
}

#[cfg(test)]
mod test {
    use std::panic;

    use super::*;

    #[test]
    #[should_panic]
    fn script_name_cannot_be_empty() {
        ScriptName::parse("".into());
    }

    #[test]
    #[should_panic]
    fn script_name_cannot_be_blank() {
        ScriptName::parse("  ".into());
    }

    #[test]
    #[should_panic]
    fn script_name_cannot_be_over_20_characters() {
        ScriptName::parse("iamlongerthan20characters".into());
    }

    #[test]
    fn script_name_cannot_contain_forbidden_characters() {
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
            let result = panic::catch_unwind(|| ScriptName::parse(case.clone()));
            assert!(
                result.is_err(),
                "{} should be a valid script name: {}",
                case,
                msg,
            );
        }
    }

    #[test]
    fn script_command_defaults_to_hi() {
        let script_cmd = Command::parse("".into());
        assert_eq!(script_cmd, Command(r#"echo 'Krabby says hi!'"#.into()))
    }

    #[test]
    fn valid_inputs_creates_valid_script() {
        let script = Script::new(
            ScriptName::parse("hello".into()),
            Command::parse("valid".into()),
        );

        assert_eq!(
            script,
            Script {
                name: ScriptName("hello".into()),
                command: Command("valid".into())
            }
        )
    }
}

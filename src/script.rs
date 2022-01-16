use serde_derive::{Deserialize, Serialize};
use std::{fmt, process::Command};

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct Script {
    pub alias: String,
    pub command: String,
}

impl Script {
    pub fn new(alias: &str, command: &str) -> Self {
        Script {
            alias: alias.to_string(),
            command: command.to_string(),
        }
    }

    fn format(&self) -> String {
        self.command.to_string()
    }

    pub fn run(&self) {
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

pub struct Shell(pub Flavor);

pub enum Flavor {
    Bash,
    Zsh,
}

impl Shell {
    pub fn script(&self) -> String {
        match &self.0 {
            Flavor::Bash | Flavor::Zsh => {
                let script = include_str!("../scripts/kb.sh");
                script.to_string()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn script_is_not_empty() {
        let cases = vec![Flavor::Bash, Flavor::Zsh];

        for case in cases {
            let shell = Shell(case);
            assert!(!shell.script().is_empty());
        }
    }
}

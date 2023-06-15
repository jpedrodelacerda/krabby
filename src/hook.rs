use crate::script::ScriptName;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(untagged)]
pub enum ProjectHook {
    ScriptArray(Vec<ScriptName>),
    Simple(String),
}

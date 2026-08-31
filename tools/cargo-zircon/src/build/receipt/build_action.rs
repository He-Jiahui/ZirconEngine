use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BuildAction {
    pub package: String,
    pub bin: Option<String>,
    pub features: Vec<String>,
}

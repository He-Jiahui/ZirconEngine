use serde::{Deserialize, Serialize};

use crate::RuntimeTargetMode;

use super::PluginModuleKind;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginModuleManifest {
    pub name: String,
    pub kind: PluginModuleKind,
    pub crate_name: String,
    #[serde(default)]
    pub target_modes: Vec<RuntimeTargetMode>,
    #[serde(default)]
    pub capabilities: Vec<String>,
}

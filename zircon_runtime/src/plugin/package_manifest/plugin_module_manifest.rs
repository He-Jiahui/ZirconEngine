use serde::{Deserialize, Serialize};

use crate::builtin::RuntimeTargetMode;
use crate::core::{InitLevel, ModuleDependencySpec};

use super::PluginModuleKind;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginModuleManifest {
    pub name: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub description: String,
    pub kind: PluginModuleKind,
    pub crate_name: String,
    #[serde(default = "default_plugin_module_init_level")]
    pub init_level: InitLevel,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub module_dependencies: Vec<ModuleDependencySpec>,
    #[serde(default)]
    pub target_modes: Vec<RuntimeTargetMode>,
    #[serde(default)]
    pub capabilities: Vec<String>,
    #[serde(default)]
    pub system_sets: Vec<String>,
    #[serde(default)]
    pub system_anchors: Vec<String>,
}

fn default_plugin_module_init_level() -> InitLevel {
    InitLevel::Post
}

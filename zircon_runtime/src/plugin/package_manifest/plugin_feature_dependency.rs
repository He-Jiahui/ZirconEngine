use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginFeatureDependency {
    pub plugin_id: String,
    pub capability: String,
    #[serde(default)]
    pub primary: bool,
}

impl PluginFeatureDependency {
    pub fn required(plugin_id: impl Into<String>, capability: impl Into<String>) -> Self {
        Self {
            plugin_id: plugin_id.into(),
            capability: capability.into(),
            primary: false,
        }
    }

    pub fn primary(plugin_id: impl Into<String>, capability: impl Into<String>) -> Self {
        Self {
            plugin_id: plugin_id.into(),
            capability: capability.into(),
            primary: true,
        }
    }
}

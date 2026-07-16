use serde::{Deserialize, Serialize};

use crate::core::framework::platform::RuntimeTargetMode;
use crate::core::{InitLevel, ModuleDependencySpec};

use super::PluginModuleKind;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginEventConsumerManifest {
    pub consumer_id: String,
    pub event_id: String,
    pub payload_schema: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub required_capability: String,
}

impl PluginEventConsumerManifest {
    pub fn new(
        consumer_id: impl Into<String>,
        event_id: impl Into<String>,
        payload_schema: impl Into<String>,
    ) -> Self {
        Self {
            consumer_id: consumer_id.into(),
            event_id: event_id.into(),
            payload_schema: payload_schema.into(),
            required_capability: String::new(),
        }
    }

    pub fn with_required_capability(mut self, capability: impl Into<String>) -> Self {
        self.required_capability = capability.into();
        self
    }
}

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
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub event_consumers: Vec<PluginEventConsumerManifest>,
}

fn default_plugin_module_init_level() -> InitLevel {
    InitLevel::Post
}

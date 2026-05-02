use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginDependencyManifest {
    pub id: String,
    pub required: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub capability: Option<String>,
}

impl PluginDependencyManifest {
    pub fn new(id: impl Into<String>, required: bool) -> Self {
        Self {
            id: id.into(),
            required,
            capability: None,
        }
    }

    pub fn with_capability(mut self, capability: impl Into<String>) -> Self {
        self.capability = Some(capability.into());
        self
    }
}

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginDependencyManifest {
    pub id: String,
    pub required: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub capability: Option<String>,
    #[serde(default)]
    pub interfaces: Vec<String>,
}

impl PluginDependencyManifest {
    pub fn new(id: impl Into<String>, required: bool) -> Self {
        Self {
            id: id.into(),
            required,
            capability: None,
            interfaces: Vec::new(),
        }
    }

    pub fn with_capability(mut self, capability: impl Into<String>) -> Self {
        self.capability = Some(capability.into());
        self
    }

    pub fn with_interface(mut self, interface_id: impl Into<String>) -> Self {
        self.interfaces.push(interface_id.into());
        self
    }

    pub fn with_interfaces<I, S>(mut self, interface_ids: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.interfaces
            .extend(interface_ids.into_iter().map(Into::into));
        self
    }
}

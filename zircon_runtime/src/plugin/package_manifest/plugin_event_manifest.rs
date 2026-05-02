use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginEventCatalogManifest {
    pub namespace: String,
    pub version: u32,
    #[serde(default)]
    pub events: Vec<PluginEventManifest>,
}

impl PluginEventCatalogManifest {
    pub fn empty(namespace: impl Into<String>, version: u32) -> Self {
        Self {
            namespace: namespace.into(),
            version,
            events: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginEventManifest {
    pub id: String,
    pub display_name: String,
    #[serde(default)]
    pub payload_schema: String,
}

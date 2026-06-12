use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginInterfaceManifest {
    pub id: String,
}

impl PluginInterfaceManifest {
    pub fn new(id: impl Into<String>) -> Self {
        Self { id: id.into() }
    }
}

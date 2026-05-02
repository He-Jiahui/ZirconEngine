use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginOptionManifest {
    pub key: String,
    pub display_name: String,
    pub value_type: String,
    pub default_value: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub required_capability: Option<String>,
}

impl PluginOptionManifest {
    pub fn new(
        key: impl Into<String>,
        display_name: impl Into<String>,
        value_type: impl Into<String>,
        default_value: impl Into<String>,
    ) -> Self {
        Self {
            key: key.into(),
            display_name: display_name.into(),
            value_type: value_type.into(),
            default_value: default_value.into(),
            required_capability: None,
        }
    }

    pub fn with_required_capability(mut self, capability: impl Into<String>) -> Self {
        self.required_capability = Some(capability.into());
        self
    }
}

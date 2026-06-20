use serde::{Deserialize, Serialize};
use zircon_runtime_interface::reflect::ReflectFieldValue;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DynamicComponent {
    pub type_path: String,
    #[serde(default)]
    pub plugin_owned: bool,
    #[serde(default)]
    pub fields: Vec<ReflectFieldValue>,
}

impl DynamicComponent {
    pub fn new(
        type_path: impl Into<String>,
        plugin_owned: bool,
        fields: Vec<ReflectFieldValue>,
    ) -> Self {
        Self {
            type_path: type_path.into(),
            plugin_owned,
            fields,
        }
    }
}

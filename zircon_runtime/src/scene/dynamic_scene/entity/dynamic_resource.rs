use serde::{Deserialize, Serialize};
use zircon_runtime_interface::reflect::ReflectFieldValue;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DynamicResource {
    pub type_path: String,
    #[serde(default)]
    pub fields: Vec<ReflectFieldValue>,
}

impl DynamicResource {
    pub fn new(type_path: impl Into<String>, fields: Vec<ReflectFieldValue>) -> Self {
        Self {
            type_path: type_path.into(),
            fields,
        }
    }
}

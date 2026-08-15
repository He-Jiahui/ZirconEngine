use serde::{Deserialize, Serialize};

/// Stable inspector-property identity carried by an inspection notification.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SceneInspectionPropertyPath {
    component_type_path: String,
    field_name: String,
}

impl SceneInspectionPropertyPath {
    pub fn new(component_type_path: impl Into<String>, field_name: impl Into<String>) -> Self {
        Self {
            component_type_path: component_type_path.into(),
            field_name: field_name.into(),
        }
    }

    pub fn component_type_path(&self) -> &str {
        &self.component_type_path
    }

    pub fn field_name(&self) -> &str {
        &self.field_name
    }
}

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReflectEditorHint {
    None,
    String,
    MultilineString,
    Bool,
    Integer,
    Unsigned,
    Scalar,
    Vec2,
    Vec3,
    Vec4,
    Enum,
    Entity,
    Resource,
    Color,
    Json,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReflectEnumOption {
    pub value: String,
    pub display_name: String,
    pub documentation: Option<String>,
}

impl ReflectEnumOption {
    pub fn new(value: impl Into<String>, display_name: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            display_name: display_name.into(),
            documentation: None,
        }
    }

    pub fn with_documentation(mut self, documentation: impl Into<String>) -> Self {
        self.documentation = Some(documentation.into());
        self
    }
}

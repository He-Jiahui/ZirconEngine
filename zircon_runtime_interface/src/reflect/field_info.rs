use serde::{Deserialize, Serialize};

use super::{ReflectEditorHint, ReflectEnumOption, ReflectNumericRange, ReflectedValue};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ReflectFieldInfo {
    pub name: String,
    pub display_name: String,
    pub value_type_path: String,
    pub editable: bool,
    pub serializable: bool,
    pub editor_visible: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_value: Option<ReflectedValue>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub numeric_range: Option<ReflectNumericRange>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub enum_options: Vec<ReflectEnumOption>,
    pub editor_hint: ReflectEditorHint,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub documentation: Option<String>,
}

impl ReflectFieldInfo {
    pub fn new(
        name: impl Into<String>,
        value_type_path: impl Into<String>,
        editor_hint: ReflectEditorHint,
    ) -> Self {
        let name = name.into();
        Self {
            display_name: name.clone(),
            name,
            value_type_path: value_type_path.into(),
            editable: true,
            serializable: true,
            editor_visible: true,
            default_value: None,
            numeric_range: None,
            enum_options: Vec::new(),
            editor_hint,
            documentation: None,
        }
    }

    pub fn with_display_name(mut self, display_name: impl Into<String>) -> Self {
        self.display_name = display_name.into();
        self
    }

    pub fn with_editable(mut self, editable: bool) -> Self {
        self.editable = editable;
        self
    }

    pub fn with_serializable(mut self, serializable: bool) -> Self {
        self.serializable = serializable;
        self
    }

    pub fn with_editor_visible(mut self, editor_visible: bool) -> Self {
        self.editor_visible = editor_visible;
        self
    }

    pub fn with_default_value(mut self, default_value: ReflectedValue) -> Self {
        self.default_value = Some(default_value);
        self
    }

    pub fn with_numeric_range(mut self, numeric_range: ReflectNumericRange) -> Self {
        self.numeric_range = Some(numeric_range);
        self
    }

    pub fn with_enum_options(mut self, enum_options: Vec<ReflectEnumOption>) -> Self {
        self.enum_options = enum_options;
        self
    }

    pub fn with_documentation(mut self, documentation: impl Into<String>) -> Self {
        self.documentation = Some(documentation.into());
        self
    }
}

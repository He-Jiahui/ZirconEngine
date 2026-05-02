use serde::{Deserialize, Serialize};

use crate::ui::component::UiValue;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiOptionDescriptor {
    pub id: String,
    pub label: String,
    pub value: UiValue,
    pub disabled: bool,
    pub special_condition: Option<String>,
}

impl UiOptionDescriptor {
    /// Creates an option descriptor with a stable id, display label, and typed value.
    pub fn new(id: impl Into<String>, label: impl Into<String>, value: UiValue) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            value,
            disabled: false,
            special_condition: None,
        }
    }

    /// Marks whether the option should reject selection.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Marks the option as a special-condition row such as a mixed inspector value.
    pub fn special_condition(mut self, condition: impl Into<String>) -> Self {
        self.special_condition = Some(condition.into());
        self
    }
}

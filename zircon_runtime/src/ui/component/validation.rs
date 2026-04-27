use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum UiValidationLevel {
    #[default]
    Normal,
    Warning,
    Error,
    Disabled,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiValidationState {
    pub level: UiValidationLevel,
    pub message: Option<String>,
}

impl UiValidationState {
    pub fn normal() -> Self {
        Self {
            level: UiValidationLevel::Normal,
            message: None,
        }
    }

    pub fn warning(message: impl Into<String>) -> Self {
        Self {
            level: UiValidationLevel::Warning,
            message: Some(message.into()),
        }
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self {
            level: UiValidationLevel::Error,
            message: Some(message.into()),
        }
    }

    pub fn disabled(message: impl Into<String>) -> Self {
        Self {
            level: UiValidationLevel::Disabled,
            message: Some(message.into()),
        }
    }

    pub fn level_name(&self) -> &'static str {
        match self.level {
            UiValidationLevel::Normal => "normal",
            UiValidationLevel::Warning => "warning",
            UiValidationLevel::Error => "error",
            UiValidationLevel::Disabled => "disabled",
        }
    }
}

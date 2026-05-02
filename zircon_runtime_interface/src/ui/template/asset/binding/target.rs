use serde::{Deserialize, Serialize};

use crate::ui::component::UiValueKind;

/// A serialized binding assignment from an expression to a runtime-owned target surface.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiBindingTargetAssignment {
    pub target: UiBindingTarget,
    pub expression: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiBindingTarget {
    pub kind: UiBindingTargetKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

impl UiBindingTarget {
    pub fn prop(name: impl Into<String>) -> Self {
        Self {
            kind: UiBindingTargetKind::Prop,
            name: Some(name.into()),
        }
    }

    pub fn class(name: impl Into<String>) -> Self {
        Self {
            kind: UiBindingTargetKind::Class,
            name: Some(name.into()),
        }
    }

    pub fn visibility() -> Self {
        Self {
            kind: UiBindingTargetKind::Visibility,
            name: None,
        }
    }

    pub fn enabled() -> Self {
        Self {
            kind: UiBindingTargetKind::Enabled,
            name: None,
        }
    }

    pub fn action_payload(name: impl Into<String>) -> Self {
        Self {
            kind: UiBindingTargetKind::ActionPayload,
            name: Some(name.into()),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiBindingTargetKind {
    Prop,
    Class,
    Visibility,
    Enabled,
    ActionPayload,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiBindingTargetSchema {
    pub target: UiBindingTarget,
    pub value_kind: UiValueKind,
}

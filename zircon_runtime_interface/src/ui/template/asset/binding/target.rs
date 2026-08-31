use serde::{Deserialize, Serialize};

use crate::ui::component::{UiValue, UiValueKind};

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(tag = "policy", rename_all = "snake_case")]
pub enum UiBindingMissingValuePolicy {
    #[default]
    Required,
    Optional,
    Default {
        value: UiValue,
    },
    Fallback {
        value: UiValue,
    },
    Error,
}

#[derive(Clone, Debug, PartialEq)]
pub enum UiBindingMissingValueResolution {
    Value(UiValue),
    Omitted,
    RequiredMissing,
    ExplicitError,
}

impl UiBindingMissingValuePolicy {
    pub fn resolve(&self, value: Option<UiValue>) -> UiBindingMissingValueResolution {
        if let Some(value) = value {
            return UiBindingMissingValueResolution::Value(value);
        }
        match self {
            Self::Required => UiBindingMissingValueResolution::RequiredMissing,
            Self::Optional => UiBindingMissingValueResolution::Omitted,
            Self::Default { value } | Self::Fallback { value } => {
                UiBindingMissingValueResolution::Value(value.clone())
            }
            Self::Error => UiBindingMissingValueResolution::ExplicitError,
        }
    }

    pub fn substitute(&self) -> Option<&UiValue> {
        match self {
            Self::Default { value } | Self::Fallback { value } => Some(value),
            Self::Required | Self::Optional | Self::Error => None,
        }
    }

    pub fn is_well_formed(&self) -> bool {
        self.substitute().is_none_or(UiValue::is_finite)
    }

    pub(crate) fn is_required(&self) -> bool {
        matches!(self, Self::Required)
    }
}

/// A serialized binding assignment from an expression to a runtime-owned target surface.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiBindingTargetAssignment {
    pub target: UiBindingTarget,
    pub expression: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiBindingTarget {
    pub kind: UiBindingTargetKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "UiBindingMissingValuePolicy::is_required"
    )]
    pub missing_policy: UiBindingMissingValuePolicy,
}

impl UiBindingTarget {
    pub fn prop(name: impl Into<String>) -> Self {
        Self {
            kind: UiBindingTargetKind::Prop,
            name: Some(name.into()),
            missing_policy: UiBindingMissingValuePolicy::Required,
        }
    }

    pub fn class(name: impl Into<String>) -> Self {
        Self {
            kind: UiBindingTargetKind::Class,
            name: Some(name.into()),
            missing_policy: UiBindingMissingValuePolicy::Required,
        }
    }

    pub fn visibility() -> Self {
        Self {
            kind: UiBindingTargetKind::Visibility,
            name: None,
            missing_policy: UiBindingMissingValuePolicy::Required,
        }
    }

    pub fn enabled() -> Self {
        Self {
            kind: UiBindingTargetKind::Enabled,
            name: None,
            missing_policy: UiBindingMissingValuePolicy::Required,
        }
    }

    pub fn action_payload(name: impl Into<String>) -> Self {
        Self {
            kind: UiBindingTargetKind::ActionPayload,
            name: Some(name.into()),
            missing_policy: UiBindingMissingValuePolicy::Required,
        }
    }

    pub fn with_missing_policy(mut self, policy: UiBindingMissingValuePolicy) -> Self {
        self.missing_policy = policy;
        self
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

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiBindingTargetSchema {
    pub target: UiBindingTarget,
    pub value_kind: UiValueKind,
}

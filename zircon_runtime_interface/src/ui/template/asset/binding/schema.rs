use std::fmt;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::ui::component::UiValueKind;

pub const UI_BINDING_SCHEMA_NAME_MAX_BYTES: usize = 256;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum UiBindingContractTerm {
    Event,
    Binding,
    Target,
    Route,
    Action,
    Command,
}

impl UiBindingContractTerm {
    pub const ALL: [Self; 6] = [
        Self::Event,
        Self::Binding,
        Self::Target,
        Self::Route,
        Self::Action,
        Self::Command,
    ];

    pub const fn schema_name(self) -> &'static str {
        match self {
            Self::Event => "event",
            Self::Binding => "binding",
            Self::Target => "target",
            Self::Route => "route",
            Self::Action => "action",
            Self::Command => "command",
        }
    }

    pub const fn definition(self) -> &'static str {
        match self {
            Self::Event => "typed input or component occurrence that triggers matching",
            Self::Binding => "compiled declaration that matches one event and owns effects",
            Self::Target => "typed state mutation endpoint owned by a binding",
            Self::Route => "dispatch destination selected for an action invocation",
            Self::Action => "named invocation and payload emitted by a binding",
            Self::Command => "host operation accepted after action routing",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum UiBindingSchemaNameKind {
    PayloadField,
    Route,
    Action,
}

impl UiBindingSchemaNameKind {
    pub const fn contract_term(self) -> Option<UiBindingContractTerm> {
        match self {
            Self::PayloadField => None,
            Self::Route => Some(UiBindingContractTerm::Route),
            Self::Action => Some(UiBindingContractTerm::Action),
        }
    }

    pub const fn diagnostic_name(self) -> &'static str {
        match self {
            Self::PayloadField => "action payload field",
            Self::Route => UiBindingContractTerm::Route.schema_name(),
            Self::Action => UiBindingContractTerm::Action.schema_name(),
        }
    }

    pub fn validate(self, value: &str) -> Result<(), UiBindingSchemaNameError> {
        if value.is_empty() {
            return Err(UiBindingSchemaNameError::Empty { kind: self });
        }
        if value.len() > UI_BINDING_SCHEMA_NAME_MAX_BYTES {
            return Err(UiBindingSchemaNameError::TooLong {
                kind: self,
                actual_bytes: value.len(),
                maximum_bytes: UI_BINDING_SCHEMA_NAME_MAX_BYTES,
            });
        }

        match self {
            Self::PayloadField => validate_payload_field(value),
            Self::Route | Self::Action => validate_dotted_name(self, value),
        }
    }
}

impl fmt::Display for UiBindingSchemaNameKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.diagnostic_name())
    }
}

#[derive(Clone, Debug, Error, PartialEq, Eq, Serialize, Deserialize)]
pub enum UiBindingSchemaNameError {
    #[error("{kind} name cannot be empty")]
    Empty { kind: UiBindingSchemaNameKind },
    #[error("{kind} name uses {actual_bytes} bytes, exceeding the {maximum_bytes}-byte limit")]
    TooLong {
        kind: UiBindingSchemaNameKind,
        actual_bytes: usize,
        maximum_bytes: usize,
    },
    #[error("{kind} name contains an empty segment at index {segment_index}")]
    EmptySegment {
        kind: UiBindingSchemaNameKind,
        segment_index: usize,
    },
    #[error("{kind} name contains invalid character `{character}` at byte {byte_index}")]
    InvalidCharacter {
        kind: UiBindingSchemaNameKind,
        character: char,
        byte_index: usize,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum UiActionPayloadFieldName {
    Additive,
    Axis,
    Channel,
    Checked,
    Committed,
    Confirm,
    Context,
    Count,
    Delta,
    Enabled,
    Fields,
    ForceFullRebuild,
    Index,
    PayloadKind,
    Primary,
    Reference,
    Scope,
    SelectionIds,
    Source,
    Subject,
    SurfaceEntity,
    Value,
    Visible,
}

impl UiActionPayloadFieldName {
    pub const ALL: [Self; 23] = [
        Self::Additive,
        Self::Axis,
        Self::Channel,
        Self::Checked,
        Self::Committed,
        Self::Confirm,
        Self::Context,
        Self::Count,
        Self::Delta,
        Self::Enabled,
        Self::Fields,
        Self::ForceFullRebuild,
        Self::Index,
        Self::PayloadKind,
        Self::Primary,
        Self::Reference,
        Self::Scope,
        Self::SelectionIds,
        Self::Source,
        Self::Subject,
        Self::SurfaceEntity,
        Self::Value,
        Self::Visible,
    ];

    pub const fn schema_name(self) -> &'static str {
        match self {
            Self::Additive => "additive",
            Self::Axis => "axis",
            Self::Channel => "channel",
            Self::Checked => "checked",
            Self::Committed => "committed",
            Self::Confirm => "confirm",
            Self::Context => "context",
            Self::Count => "count",
            Self::Delta => "delta",
            Self::Enabled => "enabled",
            Self::Fields => "fields",
            Self::ForceFullRebuild => "force_full_rebuild",
            Self::Index => "index",
            Self::PayloadKind => "payload_kind",
            Self::Primary => "primary",
            Self::Reference => "reference",
            Self::Scope => "scope",
            Self::SelectionIds => "selection_ids",
            Self::Source => "source",
            Self::Subject => "subject",
            Self::SurfaceEntity => "surface_entity",
            Self::Value => "value",
            Self::Visible => "visible",
        }
    }

    pub fn from_schema_name(value: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|field| field.schema_name() == value)
    }

    pub const fn expected_value_kind(self) -> Option<UiValueKind> {
        match self {
            Self::Additive
            | Self::Checked
            | Self::Committed
            | Self::Confirm
            | Self::Enabled
            | Self::ForceFullRebuild
            | Self::Visible => Some(UiValueKind::Bool),
            Self::Count | Self::Delta | Self::Index | Self::SurfaceEntity => Some(UiValueKind::Int),
            _ => None,
        }
    }
}

fn validate_payload_field(value: &str) -> Result<(), UiBindingSchemaNameError> {
    for (byte_index, character) in value.char_indices() {
        if !(character.is_ascii_lowercase() || character.is_ascii_digit() || character == '_') {
            return Err(UiBindingSchemaNameError::InvalidCharacter {
                kind: UiBindingSchemaNameKind::PayloadField,
                character,
                byte_index,
            });
        }
    }
    Ok(())
}

fn validate_dotted_name(
    kind: UiBindingSchemaNameKind,
    value: &str,
) -> Result<(), UiBindingSchemaNameError> {
    for (segment_index, segment) in value.split('.').enumerate() {
        if segment.is_empty() {
            return Err(UiBindingSchemaNameError::EmptySegment {
                kind,
                segment_index,
            });
        }
    }
    for (byte_index, character) in value.char_indices() {
        if !(character.is_ascii_alphanumeric() || matches!(character, '.' | '_' | '-')) {
            return Err(UiBindingSchemaNameError::InvalidCharacter {
                kind,
                character,
                byte_index,
            });
        }
    }
    Ok(())
}

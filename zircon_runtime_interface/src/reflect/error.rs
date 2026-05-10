use std::fmt;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum ReflectError {
    InvalidTypePath {
        type_path: String,
        reason: String,
    },
    UnknownType {
        type_path: String,
    },
    AmbiguousShortTypePath {
        short_type_path: String,
    },
    DuplicateTypePath {
        type_path: String,
    },
    NoComponentAdapter {
        type_path: String,
    },
    NoResourceAdapter {
        type_path: String,
    },
    MissingEntity {
        entity: u64,
    },
    MissingComponent {
        entity: u64,
        type_path: String,
    },
    MissingResource {
        type_path: String,
    },
    UnknownField {
        type_path: String,
        field_name: String,
    },
    NonEditableField {
        type_path: String,
        field_name: String,
    },
    TypeMismatch {
        type_path: String,
        field_name: String,
        expected: String,
        actual: String,
    },
    UnsupportedConversion {
        source: String,
        target: String,
    },
    AddressKindMismatch {
        expected: String,
        actual: String,
    },
    InvalidRegistration {
        type_path: String,
        reason: String,
    },
}

impl fmt::Display for ReflectError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidTypePath { type_path, reason } => write!(
                formatter,
                "invalid reflected type path `{type_path}`: {reason}"
            ),
            Self::UnknownType { type_path } => {
                write!(formatter, "unknown reflected type `{type_path}`")
            }
            Self::AmbiguousShortTypePath { short_type_path } => write!(
                formatter,
                "ambiguous reflected short type path `{short_type_path}`"
            ),
            Self::DuplicateTypePath { type_path } => {
                write!(formatter, "duplicate reflected type path `{type_path}`")
            }
            Self::NoComponentAdapter { type_path } => write!(
                formatter,
                "reflected type `{type_path}` has no component adapter"
            ),
            Self::NoResourceAdapter { type_path } => write!(
                formatter,
                "reflected type `{type_path}` has no resource adapter"
            ),
            Self::MissingEntity { entity } => {
                write!(formatter, "reflected entity `{entity}` does not exist")
            }
            Self::MissingComponent { entity, type_path } => write!(
                formatter,
                "entity `{entity}` is missing reflected component `{type_path}`"
            ),
            Self::MissingResource { type_path } => {
                write!(formatter, "reflected resource `{type_path}` does not exist")
            }
            Self::UnknownField {
                type_path,
                field_name,
            } => write!(
                formatter,
                "reflected type `{type_path}` has no field `{field_name}`"
            ),
            Self::NonEditableField {
                type_path,
                field_name,
            } => write!(
                formatter,
                "reflected field `{field_name}` on `{type_path}` is not editable"
            ),
            Self::TypeMismatch {
                type_path,
                field_name,
                expected,
                actual,
            } => write!(
                formatter,
                "reflected field `{field_name}` on `{type_path}` expected `{expected}` but received `{actual}`"
            ),
            Self::UnsupportedConversion { source, target } => write!(
                formatter,
                "unsupported reflected conversion from `{source}` to `{target}`"
            ),
            Self::AddressKindMismatch { expected, actual } => write!(
                formatter,
                "reflected address kind mismatch: expected `{expected}` but received `{actual}`"
            ),
            Self::InvalidRegistration { type_path, reason } => write!(
                formatter,
                "invalid reflected registration `{type_path}`: {reason}"
            ),
        }
    }
}

impl std::error::Error for ReflectError {}

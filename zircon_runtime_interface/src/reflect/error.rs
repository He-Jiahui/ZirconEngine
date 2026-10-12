//! 反射目录、值转换和世界适配器共享这组可序列化错误；结构化字段让调用端匹配失败类别时不必解析显示文本。
use std::fmt;

use serde::{Deserialize, Serialize};

use crate::world_sync::LifecycleTargetChange;

/// 反射目录、值转换和世界适配器用这些变体区分契约失败，调用端可直接匹配错误类型。
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum ReflectError {
    LifecycleTargetReplaced {
        operation: String,
        target: LifecycleTargetChange,
    },
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
    NonRemovableComponent {
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
    InvalidFieldRegistration {
        type_path: String,
        field_name: String,
        reason: String,
    },
    InvalidValue {
        type_path: String,
        field_name: String,
        reason: String,
    },
}

impl ReflectError {
    /// A callback retired the producer's target; callers must stop old-target work.
    pub fn lifecycle_target_change(&self) -> Option<LifecycleTargetChange> {
        match self {
            Self::LifecycleTargetReplaced { target, .. } => Some(*target),
            _ => None,
        }
    }
}

impl fmt::Display for ReflectError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LifecycleTargetReplaced { operation, target } => write!(
                formatter,
                "{operation} cancelled because a lifecycle callback replaced {target}"
            ),
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
            Self::NonRemovableComponent { type_path } => {
                write!(formatter, "reflected component `{type_path}` cannot be removed")
            }
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
            Self::InvalidFieldRegistration {
                type_path,
                field_name,
                reason,
            } => write!(
                formatter,
                "invalid reflected field `{field_name}` on `{type_path}`: {reason}"
            ),
            Self::InvalidValue {
                type_path,
                field_name,
                reason,
            } => write!(
                formatter,
                "invalid reflected value for field `{field_name}` on `{type_path}`: {reason}"
            ),
        }
    }
}

impl std::error::Error for ReflectError {}

#[cfg(test)]
mod lifecycle_cancellation_tests {
    use super::*;

    #[test]
    fn reflected_lifecycle_cancellation_preserves_kind_and_target_on_the_wire() {
        for (target, wire_target) in [
            (LifecycleTargetChange::World, "world"),
            (LifecycleTargetChange::Schedule, "schedule"),
        ] {
            let error = ReflectError::LifecycleTargetReplaced {
                operation: "remove component".into(),
                target,
            };
            let encoded = serde_json::to_value(&error).unwrap();
            assert_eq!(
                encoded,
                serde_json::json!({
                    "kind": "LifecycleTargetReplaced",
                    "operation": "remove component",
                    "target": wire_target,
                })
            );
            let decoded: ReflectError = serde_json::from_value(encoded).unwrap();
            assert_eq!(decoded, error);
            assert_eq!(decoded.lifecycle_target_change(), Some(target));
        }
    }

    #[test]
    fn ordinary_reflection_conversion_keeps_its_existing_wire_contract() {
        let error = ReflectError::UnsupportedConversion {
            source: "authored conversion".into(),
            target: "Name.value".into(),
        };
        let encoded = serde_json::to_value(&error).unwrap();
        assert_eq!(
            encoded,
            serde_json::json!({
                "kind": "UnsupportedConversion",
                "source": "authored conversion",
                "target": "Name.value",
            })
        );
        assert_eq!(
            serde_json::from_value::<ReflectError>(encoded).unwrap(),
            error
        );
        assert_eq!(error.lifecycle_target_change(), None);
    }
}

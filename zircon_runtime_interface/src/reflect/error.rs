//! 反射目录、值转换和世界适配器共享这组可序列化错误；结构化字段让调用端匹配失败类别时不必解析显示文本。
use std::fmt;

use serde::{Deserialize, Serialize};

use crate::world_sync::LifecycleTargetChange;

/// 反射目录、值转换和世界适配器用这些变体区分契约失败，调用端可直接匹配错误类型。
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum ReflectError {
    /// 生命周期回调替换了当前操作所依赖的目标，调用端应停止旧目标上的后续工作。
    LifecycleTargetReplaced {
        /// 被取消操作的描述文本。
        operation: String,
        /// 发生替换的 World 或 Schedule 目标。
        target: LifecycleTargetChange,
    },
    /// 类型路径未通过共享路径语法校验。
    InvalidTypePath {
        /// 无效的输入路径。
        type_path: String,
        /// 校验器给出的失败原因。
        reason: String,
    },
    /// 请求路径未找到已接纳的类型注册。
    UnknownType {
        /// 未找到注册的类型路径。
        type_path: String,
    },
    /// 仅凭短类型路径无法唯一确定注册项。
    AmbiguousShortTypePath {
        /// 发生冲突的短路径。
        short_type_path: String,
    },
    /// 注册目录拒绝重复的完整类型路径。
    DuplicateTypePath {
        /// 已占用的类型路径。
        type_path: String,
    },
    /// 目标组件类型没有可执行读写的组件适配器。
    NoComponentAdapter {
        /// 缺少适配器的类型路径。
        type_path: String,
    },
    /// 目标资源类型没有可执行读写的资源适配器。
    NoResourceAdapter {
        /// 缺少适配器的类型路径。
        type_path: String,
    },
    /// 组件地址引用的实体不存在。
    MissingEntity {
        /// 未找到的实体标识。
        entity: u64,
    },
    /// 实体存在，但不含请求类型的组件。
    MissingComponent {
        /// 组件所在实体。
        entity: u64,
        /// 未找到的组件类型路径。
        type_path: String,
    },
    /// 请求移除的组件被声明为不可移除。
    NonRemovableComponent {
        /// 不允许移除的组件类型路径。
        type_path: String,
    },
    /// 资源地址引用的注册资源当前不存在。
    MissingResource {
        /// 未找到的资源类型路径。
        type_path: String,
    },
    /// 类型注册中没有可由请求解析的字段。
    UnknownField {
        /// 字段所属类型。
        type_path: String,
        /// 请求或适配器报告的字段名。
        field_name: String,
    },
    /// 字段存在，但其元数据禁止编辑写入。
    NonEditableField {
        /// 字段所属类型。
        type_path: String,
        /// 不可编辑字段名。
        field_name: String,
    },
    /// 输入反射值不能转换为字段声明的 Rust 值类型。
    TypeMismatch {
        /// 字段所属类型。
        type_path: String,
        /// 转换失败字段名。
        field_name: String,
        /// 转换器要求的值类别或范围。
        expected: String,
        /// 实际收到的值类别或范围。
        actual: String,
    },
    /// 请求的源值与目标值组合没有已实现的转换。
    UnsupportedConversion {
        /// 来源描述。
        source: String,
        /// 目标描述。
        target: String,
    },
    /// 操作要求的对象地址种类与收到的种类不一致。
    AddressKindMismatch {
        /// 操作要求的地址种类。
        expected: String,
        /// 实际收到的地址种类。
        actual: String,
    },
    /// 类型注册未通过目录或运行时注册校验。
    InvalidRegistration {
        /// 被拒绝注册的类型路径。
        type_path: String,
        /// 注册校验失败原因。
        reason: String,
    },
    /// 注册中的字段元数据未通过字段级校验。
    InvalidFieldRegistration {
        /// 字段所属类型路径。
        type_path: String,
        /// 无效字段名。
        field_name: String,
        /// 字段校验失败原因。
        reason: String,
    },
    /// 字段的反射值未满足运行时字段值约束。
    InvalidValue {
        /// 字段所属类型路径。
        type_path: String,
        /// 值无效字段名。
        field_name: String,
        /// 值校验失败原因。
        reason: String,
    },
}

impl ReflectError {
    /// A callback retired the producer's target; callers must stop old-target work.
    /// 返回生命周期取消变体中的替换目标；其它错误返回 `None`。
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

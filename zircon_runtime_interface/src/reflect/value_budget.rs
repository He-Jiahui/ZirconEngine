use serde::{Deserialize, Serialize};
use thiserror::Error;

/// 一次反射值验证的调用方配额，覆盖反射容器及嵌入的 JSON 子树；入口按信任边界选择上限，本类型不提供隐式默认值。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReflectValueBudget {
    max_depth: usize,
    max_nodes: usize,
    max_string_bytes: usize,
    max_container_entries: usize,
}

impl ReflectValueBudget {
    pub const fn new(
        max_depth: usize,
        max_nodes: usize,
        max_string_bytes: usize,
        max_container_entries: usize,
    ) -> Self {
        Self {
            max_depth,
            max_nodes,
            max_string_bytes,
            max_container_entries,
        }
    }

    pub const fn max_depth(self) -> usize {
        self.max_depth
    }

    pub const fn max_nodes(self) -> usize {
        self.max_nodes
    }

    pub const fn max_string_bytes(self) -> usize {
        self.max_string_bytes
    }

    pub const fn max_container_entries(self) -> usize {
        self.max_container_entries
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReflectValueFloatKind {
    Scalar,
    Vec2,
    Vec3,
    Vec4,
    Quaternion,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReflectValueBudgetDimension {
    Depth,
    Nodes,
    StringBytes,
}

#[derive(Clone, Copy, Debug, Error, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReflectValueValidationError {
    #[error("reflected value {dimension:?} accounting overflowed")]
    BudgetArithmeticOverflow {
        dimension: ReflectValueBudgetDimension,
    },
    #[error("reflected value depth {actual} exceeds the configured limit {maximum}")]
    DepthExceeded { actual: usize, maximum: usize },
    #[error("reflected value node count {actual} exceeds the configured limit {maximum}")]
    NodeBudgetExceeded { actual: usize, maximum: usize },
    #[error("reflected value string bytes {actual} exceed the configured limit {maximum}")]
    StringBudgetExceeded { actual: usize, maximum: usize },
    #[error(
        "reflected value container has {actual} entries, exceeding the configured limit {maximum}"
    )]
    ContainerEntriesExceeded { actual: usize, maximum: usize },
    #[error("reflected {kind:?} component {component} is not finite")]
    NonFiniteFloat {
        kind: ReflectValueFloatKind,
        component: usize,
    },
}

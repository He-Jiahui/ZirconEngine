//! 集中定义物理 provider 生命周期、描述符验证、容量和步进失败的结构化错误类型及其显示文本。

use std::error::Error;
use std::fmt;

use zircon_runtime::core::math::Real;

/// 错误中标识的对象类别；与具体后端错误分开，便于调用方按对象类型处理句柄和容量失败。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PhysicsBackendObjectKind {
    Shape,
    Body,
    Constraint,
}

/// 后端操作失败的分类；调用方可匹配错误种类，detail 保留 provider 或描述符的具体原因。
#[derive(Clone, Debug, PartialEq)]
pub enum PhysicsBackendError {
    Initialization {
        backend: &'static str,
        detail: String,
    },
    InvalidHandle {
        kind: PhysicsBackendObjectKind,
        raw: u64,
    },
    InvalidDescriptor {
        kind: PhysicsBackendObjectKind,
        detail: String,
    },
    ObjectInUse {
        kind: PhysicsBackendObjectKind,
        raw: u64,
    },
    CapacityExhausted {
        kind: PhysicsBackendObjectKind,
    },
    InvalidStepSeconds {
        value: Real,
    },
    StepFailed {
        backend: &'static str,
        code: u32,
    },
    Unsupported {
        backend: &'static str,
        operation: &'static str,
        detail: &'static str,
    },
}

impl fmt::Display for PhysicsBackendError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Initialization { backend, detail } => {
                write!(
                    formatter,
                    "{backend} backend initialization failed: {detail}"
                )
            }
            Self::InvalidHandle { kind, raw } => {
                write!(formatter, "invalid {kind:?} handle {raw:#018x}")
            }
            Self::InvalidDescriptor { kind, detail } => {
                write!(formatter, "invalid {kind:?} descriptor: {detail}")
            }
            Self::ObjectInUse { kind, raw } => {
                write!(formatter, "{kind:?} handle {raw:#018x} is still in use")
            }
            Self::CapacityExhausted { kind } => {
                write!(formatter, "{kind:?} handle capacity is exhausted")
            }
            Self::InvalidStepSeconds { value } => {
                write!(
                    formatter,
                    "physics step must be finite and positive, got {value}"
                )
            }
            Self::StepFailed { backend, code } => {
                write!(
                    formatter,
                    "{backend} backend step failed with code {code:#010x}"
                )
            }
            Self::Unsupported {
                backend,
                operation,
                detail,
            } => write!(
                formatter,
                "{backend} backend does not support {operation}: {detail}"
            ),
        }
    }
}

impl Error for PhysicsBackendError {}

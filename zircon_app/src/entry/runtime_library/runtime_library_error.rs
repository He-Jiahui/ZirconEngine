//! App 与动态 Runtime 边界使用的错误分类和诊断文本。
//! 调用方可展示文本；若按 kind 分流，应核实经过哪些包装层。

use std::error::Error;
use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// 区分普通调用、可选能力不可用和跨 ABI 协议违规的内部诊断类别。
pub(crate) enum RuntimeLibraryErrorKind {
    General,
    CapabilityUnavailable,
    ProtocolViolation,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct RuntimeLibraryError {
    kind: RuntimeLibraryErrorKind,
    message: String,
}

impl RuntimeLibraryError {
    pub(crate) fn new(message: impl Into<String>) -> Self {
        Self {
            kind: RuntimeLibraryErrorKind::General,
            message: message.into(),
        }
    }

    pub(crate) fn protocol_violation(message: impl Into<String>) -> Self {
        Self {
            kind: RuntimeLibraryErrorKind::ProtocolViolation,
            message: message.into(),
        }
    }

    pub(crate) fn capability_unavailable(message: impl Into<String>) -> Self {
        Self {
            kind: RuntimeLibraryErrorKind::CapabilityUnavailable,
            message: message.into(),
        }
    }

    pub(crate) const fn kind(&self) -> RuntimeLibraryErrorKind {
        self.kind
    }

    /// 将清理阶段错误附于主错误，同时保留主错误的分类供上层判定。
    pub(crate) fn with_cleanup_failure(self, cleanup: &RuntimeLibraryError) -> Self {
        Self {
            kind: self.kind,
            message: format!("{}; cleanup also failed: {cleanup}", self.message),
        }
    }
}

impl fmt::Display for RuntimeLibraryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl Error for RuntimeLibraryError {}

impl From<zircon_runtime_host::foreign_output::RuntimeForeignOutputError> for RuntimeLibraryError {
    fn from(error: zircon_runtime_host::foreign_output::RuntimeForeignOutputError) -> Self {
        use zircon_runtime_host::foreign_output::RuntimeForeignOutputErrorKind;

        match error.kind() {
            RuntimeForeignOutputErrorKind::RuntimeCall => Self::new(error.to_string()),
            RuntimeForeignOutputErrorKind::ProtocolViolation => {
                Self::protocol_violation(error.to_string())
            }
        }
    }
}

#[cfg(test)]
#[path = "tests/runtime_library_error.rs"]
mod tests;

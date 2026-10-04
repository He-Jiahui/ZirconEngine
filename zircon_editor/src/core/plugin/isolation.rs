//! Panic containment for editor-plugin registration and lifecycle callbacks.

use std::any::Any;
use std::fmt;
use std::panic::{catch_unwind, AssertUnwindSafe};

#[derive(Clone, Debug, Eq, PartialEq)]
/// 插件边界失败的宿主诊断，记录包和操作以便隔离故障；捕获 panic 不保证插件此前副作用回滚。
pub struct EditorPluginBoundaryFailure {
    package_id: String,
    operation: String,
    detail: String,
}

impl EditorPluginBoundaryFailure {
    fn rejected(
        package_id: impl Into<String>,
        operation: impl Into<String>,
        detail: impl Into<String>,
    ) -> Self {
        Self {
            package_id: package_id.into(),
            operation: operation.into(),
            detail: detail.into(),
        }
    }
}

impl fmt::Display for EditorPluginBoundaryFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "editor plugin `{}` {} failed: {}",
            self.package_id, self.operation, self.detail
        )
    }
}

impl std::error::Error for EditorPluginBoundaryFailure {}

/// Converts a plugin callback failure or panic into a recoverable host diagnostic.
pub fn run_editor_plugin_boundary<T>(
    package_id: &str,
    operation: &str,
    callback: impl FnOnce() -> Result<T, String>,
) -> Result<T, EditorPluginBoundaryFailure> {
    match catch_unwind(AssertUnwindSafe(callback)) {
        Ok(Ok(value)) => Ok(value),
        Ok(Err(detail)) => Err(EditorPluginBoundaryFailure::rejected(
            package_id, operation, detail,
        )),
        Err(payload) => Err(EditorPluginBoundaryFailure::rejected(
            package_id,
            operation,
            format!("panic: {}", panic_payload_message(payload)),
        )),
    }
}

fn panic_payload_message(payload: Box<dyn Any + Send>) -> String {
    match payload.downcast::<String>() {
        Ok(message) => *message,
        Err(payload) => match payload.downcast::<&'static str>() {
            Ok(message) => (*message).to_string(),
            Err(_) => "non-string payload".to_string(),
        },
    }
}

#[cfg(test)]
#[path = "tests/isolation.rs"]
mod tests;

//! Panic containment for editor-plugin registration and lifecycle callbacks.

use std::any::Any;
use std::fmt;
use std::panic::{catch_unwind, AssertUnwindSafe};

#[derive(Clone, Debug, Eq, PartialEq)]
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
mod tests {
    use super::run_editor_plugin_boundary;

    #[test]
    fn callback_error_stays_a_plugin_diagnostic() {
        let failure = run_editor_plugin_boundary("plugin.sample", "register", || {
            Err::<(), _>("invalid contribution".to_string())
        })
        .expect_err("plugin rejection should not escape the host boundary");

        assert_eq!(
            failure.to_string(),
            "editor plugin `plugin.sample` register failed: invalid contribution"
        );
    }

    #[test]
    fn callback_panic_stays_a_plugin_diagnostic() {
        let failure =
            run_editor_plugin_boundary("plugin.sample", "register", || -> Result<(), String> {
                panic!("fixture panic")
            })
            .expect_err("plugin panic should not escape the host boundary");

        assert_eq!(
            failure.to_string(),
            "editor plugin `plugin.sample` register failed: panic: fixture panic"
        );
    }
}

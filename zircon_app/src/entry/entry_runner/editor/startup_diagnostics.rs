use std::error::Error;
use std::fmt::{self, Display, Formatter};

#[derive(Debug)]
pub(super) struct EditorStartupDiagnosticError {
    component: &'static str,
    requested: String,
    cause: String,
    recovery: &'static str,
}

impl Display for EditorStartupDiagnosticError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "editor startup diagnostic: component={} requested={} cause={} recovery={}",
            self.component, self.requested, self.cause, self.recovery
        )
    }
}

impl Error for EditorStartupDiagnosticError {}

pub(super) fn editor_startup_diagnostic_error(
    component: &'static str,
    requested: impl Into<String>,
    cause: impl Into<String>,
    recovery: &'static str,
) -> EditorStartupDiagnosticError {
    EditorStartupDiagnosticError {
        component,
        requested: requested.into(),
        cause: cause.into(),
        recovery,
    }
}

pub(super) fn finish_editor_host<T, E>(
    requested: &str,
    host_result: Result<T, Box<dyn Error>>,
    teardown_failure: Option<E>,
) -> Result<T, Box<dyn Error>>
where
    E: Display,
{
    match (host_result, teardown_failure) {
        (Ok(result), None) => Ok(result),
        (Ok(_), Some(teardown_failure)) => Err(editor_startup_diagnostic_error(
            "runtime_session",
            requested,
            format!("runtime session teardown failed: {teardown_failure}"),
            "verify the runtime session lifecycle and staged runtime ABI before retrying zircon_editor",
        )
        .into()),
        (Err(host_error), None) => Err(host_error),
        (Err(host_error), Some(teardown_failure)) => Err(editor_startup_diagnostic_error(
            "editor_host",
            requested,
            format!(
                "editor host failed: {host_error}; runtime session teardown also failed: {teardown_failure}"
            ),
            "inspect both the editor host and runtime session failures before retrying zircon_editor",
        )
        .into()),
    }
}

pub(super) fn editor_host_startup_error(
    requested: &str,
    source: Box<dyn Error>,
) -> EditorStartupDiagnosticError {
    editor_startup_diagnostic_error(
        "editor_host",
        requested,
        format!("editor host execution failed: {source}"),
        "verify the requested project or view and the staged editor assets before retrying zircon_editor",
    )
}

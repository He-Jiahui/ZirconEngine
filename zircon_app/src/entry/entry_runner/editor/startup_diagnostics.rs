use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::path::Path;
use zircon_runtime::asset::project::ProjectPaths;

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

pub(super) fn editor_startup_argument_error(
    args: &[String],
    source: Box<dyn Error>,
) -> Box<dyn Error> {
    let requested = editor_startup_argument_summary(args);
    let cause = redact_editor_startup_argument_cause(args, source.to_string());
    editor_startup_diagnostic_error(
        "editor_app",
        requested,
        cause,
        "provide one valid editor startup mode and run zircon_editor --help to inspect supported arguments",
    )
    .into()
}

pub(super) fn editor_operation_startup_error(
    requested: &str,
    source: impl Display,
) -> EditorStartupDiagnosticError {
    editor_startup_diagnostic_error(
        "editor_operation",
        requested,
        format!("editor operation startup failed: {source}"),
        "verify the staged runtime ABI, editor operation registrations, and selected operation before retrying zircon_editor",
    )
}

pub(super) fn editor_automation_startup_error(
    requested: &str,
    source: impl Display,
) -> EditorStartupDiagnosticError {
    editor_startup_diagnostic_error(
        "editor_automation",
        requested,
        format!("project automation failed: {source}"),
        "verify the project path, automation JSON, editor bindings, and staged runtime before retrying zircon_editor",
    )
}

pub(super) fn finish_editor_operation<T, E>(
    operation_result: Result<T, Box<dyn Error>>,
    teardown_failure: Option<E>,
) -> Result<T, Box<dyn Error>>
where
    E: Display,
{
    match (operation_result, teardown_failure) {
        (Ok(response), None) => Ok(response),
        (Ok(_), Some(teardown_failure)) => {
            Err(format!("runtime session teardown failed: {teardown_failure}").into())
        }
        (Err(operation_error), None) => Err(operation_error),
        (Err(operation_error), Some(teardown_failure)) => Err(format!(
            "editor operation failed: {operation_error}; runtime session teardown also failed: {teardown_failure}"
        )
        .into()),
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

pub(super) fn editor_startup_argument_summary(args: &[String]) -> String {
    if args.is_empty() {
        return "<empty>".to_string();
    }

    let mut redact_next = false;
    let mut display_path_next = false;
    args.iter()
        .map(|argument| {
            if redact_next {
                redact_next = false;
                return "<redacted>".to_string();
            }
            if argument.starts_with("--args=") {
                display_path_next = false;
                return "--args=<redacted>".to_string();
            }
            if argument == "--args" {
                display_path_next = false;
                redact_next = true;
                return argument.clone();
            }
            if display_path_next {
                display_path_next = false;
                return editor_startup_path_display(argument);
            }
            display_path_next = editor_startup_argument_is_path_flag(argument);
            argument.clone()
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn redact_editor_startup_argument_cause(args: &[String], mut cause: String) -> String {
    let mut redact_next = false;
    let mut display_path_next = false;
    for argument in args {
        if redact_next {
            if !argument.is_empty() {
                cause = cause.replace(argument, "<redacted>");
            }
            redact_next = false;
            continue;
        }
        if argument == "--args" {
            display_path_next = false;
            redact_next = true;
        } else if argument.starts_with("--args=") {
            display_path_next = false;
            cause = cause.replace(argument, "--args=<redacted>");
        } else if display_path_next {
            cause = cause.replace(argument, &editor_startup_path_display(argument));
            display_path_next = false;
        } else {
            display_path_next = editor_startup_argument_is_path_flag(argument);
        }
    }
    cause
}

fn editor_startup_argument_is_path_flag(argument: &str) -> bool {
    matches!(argument, "--project" | "--automation" | "--location")
}

fn editor_startup_path_display(argument: &str) -> String {
    ProjectPaths::display_path(Path::new(argument))
        .display()
        .to_string()
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

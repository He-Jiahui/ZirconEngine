use std::process::{Child, Command};

use super::ExportProcessTerminationError;

pub(in crate::ui::host) fn configure_process_tree_cancellation(process: &mut Command) {
    configure_platform_process_tree(process);
}

pub(in crate::ui::host) struct ProcessTreeTermination {
    pub(in crate::ui::host) diagnostic: String,
    pub(in crate::ui::host) succeeded: bool,
    pub(in crate::ui::host) error: Option<ExportProcessTerminationError>,
}

pub(in crate::ui::host) fn terminate_process_tree(
    child: &mut Child,
    label: &str,
) -> ProcessTreeTermination {
    let child_id = child.id();
    let mut diagnostics = Vec::new();
    let mut primary_error = None;
    if let Some(result) = terminate_platform_process_tree(child_id, label) {
        diagnostics.push(result.diagnostic);
        primary_error = result.error;
        if result.succeeded {
            return ProcessTreeTermination {
                diagnostic: diagnostics.join("\n"),
                succeeded: true,
                error: None,
            };
        }
    }
    let (diagnostic, succeeded, error) = match child.kill() {
        Ok(()) => (
            format!("{label} cancelled; process was terminated"),
            true,
            None,
        ),
        Err(source) => (
            format!("{label} cancellation requested but termination failed: {source}"),
            false,
            Some(ExportProcessTerminationError::FallbackKill {
                primary: primary_error.map(Box::new),
                source,
            }),
        ),
    };
    diagnostics.push(diagnostic);
    ProcessTreeTermination {
        diagnostic: diagnostics.join("\n"),
        succeeded,
        error,
    }
}

#[cfg(unix)]
fn configure_platform_process_tree(process: &mut Command) {
    use std::os::unix::process::CommandExt;

    process.process_group(0);
}

#[cfg(not(unix))]
fn configure_platform_process_tree(_process: &mut Command) {}

struct PlatformProcessTreeTermination {
    diagnostic: String,
    succeeded: bool,
    error: Option<ExportProcessTerminationError>,
}

#[cfg(windows)]
fn terminate_platform_process_tree(
    child_id: u32,
    label: &str,
) -> Option<PlatformProcessTreeTermination> {
    let output = Command::new("taskkill")
        .args(platform_process_tree_termination_args(child_id))
        .output();
    Some(match output {
        Ok(output) if output.status.success() => PlatformProcessTreeTermination {
            diagnostic: format!("{label} cancelled; process tree was terminated"),
            succeeded: true,
            error: None,
        },
        Ok(output) => PlatformProcessTreeTermination {
            diagnostic: format!(
                "{label} cancellation requested but taskkill failed with status {:?}: {}{}",
                output.status.code(),
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            ),
            succeeded: false,
            error: Some(ExportProcessTerminationError::CommandExit {
                program: "taskkill",
                status_code: output.status.code(),
                stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
                stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
            }),
        },
        Err(source) => PlatformProcessTreeTermination {
            diagnostic: format!("{label} cancellation requested but taskkill failed: {source}"),
            succeeded: false,
            error: Some(ExportProcessTerminationError::CommandSpawn {
                program: "taskkill",
                source,
            }),
        },
    })
}

#[cfg(all(unix, not(windows)))]
fn terminate_platform_process_tree(
    child_id: u32,
    label: &str,
) -> Option<PlatformProcessTreeTermination> {
    let output = Command::new("kill")
        .args(platform_process_tree_termination_args(child_id))
        .output();
    Some(match output {
        Ok(output) if output.status.success() => PlatformProcessTreeTermination {
            diagnostic: format!("{label} cancelled; process group was terminated"),
            succeeded: true,
            error: None,
        },
        Ok(output) => PlatformProcessTreeTermination {
            diagnostic: format!(
                "{label} cancellation requested but process-group kill failed with status {:?}: {}{}",
                output.status.code(),
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            ),
            succeeded: false,
            error: Some(ExportProcessTerminationError::CommandExit {
                program: "kill",
                status_code: output.status.code(),
                stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
                stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
            }),
        },
        Err(source) => PlatformProcessTreeTermination {
            diagnostic: format!(
                "{label} cancellation requested but process-group kill failed: {source}"
            ),
            succeeded: false,
            error: Some(ExportProcessTerminationError::CommandSpawn {
                program: "kill",
                source,
            }),
        },
    })
}

#[cfg(not(any(windows, unix)))]
fn terminate_platform_process_tree(
    _child_id: u32,
    _label: &str,
) -> Option<PlatformProcessTreeTermination> {
    None
}

#[cfg(windows)]
fn platform_process_tree_termination_args(child_id: u32) -> Vec<String> {
    vec![
        "/PID".to_string(),
        child_id.to_string(),
        "/T".to_string(),
        "/F".to_string(),
    ]
}

#[cfg(all(unix, not(windows)))]
fn platform_process_tree_termination_args(child_id: u32) -> Vec<String> {
    vec!["-KILL".to_string(), format!("-{child_id}")]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(windows)]
    #[test]
    fn process_tree_termination_args_use_windows_tree_kill() {
        assert_eq!(
            platform_process_tree_termination_args(42),
            vec!["/PID", "42", "/T", "/F"]
        );
    }

    #[cfg(all(unix, not(windows)))]
    #[test]
    fn process_tree_termination_args_use_unix_group_kill() {
        assert_eq!(
            platform_process_tree_termination_args(42),
            vec!["-KILL", "-42"]
        );
    }
}

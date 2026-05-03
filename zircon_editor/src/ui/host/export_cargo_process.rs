use std::io::Read;
use std::path::Path;
use std::process::{Child, Command, ExitStatus, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use super::editor_manager_plugins_export::EditorExportCargoInvocation;

const CARGO_PROCESS_CANCEL_POLL_INTERVAL: Duration = Duration::from_millis(100);

pub(in crate::ui::host) fn invoke_cargo_process(
    cargo: String,
    args: Vec<String>,
    current_dir: Option<&Path>,
    cancel_requested: Option<&AtomicBool>,
    label: &str,
) -> Result<EditorExportCargoInvocation, String> {
    let mut command = Vec::with_capacity(args.len() + 1);
    command.push(cargo.clone());
    command.extend(args.clone());

    if cancel_requested.is_some_and(cancellation_requested) {
        return Ok(cancelled_invocation(
            command,
            format!("{label} cancelled before Cargo started"),
        ));
    }

    let mut process = Command::new(&cargo);
    process
        .args(&args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    if let Some(current_dir) = current_dir {
        process.current_dir(current_dir);
    }
    configure_cargo_process_for_tree_cancellation(&mut process);

    let mut child = process
        .spawn()
        .map_err(|error| format!("failed to invoke cargo for {label}: {error}"))?;
    let stdout_reader = child.stdout.take().map(read_pipe_to_string);
    let stderr_reader = child.stderr.take().map(read_pipe_to_string);

    loop {
        if cancel_requested.is_some_and(cancellation_requested) {
            let kill_diagnostic = terminate_cargo_process_tree(&mut child, label);
            let status = child
                .wait()
                .map_err(|error| format!("failed to collect cancelled Cargo output: {error}"))?;
            let mut invocation = invocation_from_status(
                command,
                status,
                collect_reader_output(stdout_reader, label, "stdout")?,
                collect_reader_output(stderr_reader, label, "stderr")?,
            );
            invocation.success = false;
            if invocation.stderr.is_empty() {
                invocation.stderr = kill_diagnostic;
            } else {
                invocation.stderr.push('\n');
                invocation.stderr.push_str(&kill_diagnostic);
            }
            return Ok(invocation);
        }

        match child
            .try_wait()
            .map_err(|error| format!("failed to poll cargo process for {label}: {error}"))?
        {
            Some(status) => {
                return Ok(invocation_from_status(
                    command,
                    status,
                    collect_reader_output(stdout_reader, label, "stdout")?,
                    collect_reader_output(stderr_reader, label, "stderr")?,
                ));
            }
            None => thread::sleep(CARGO_PROCESS_CANCEL_POLL_INTERVAL),
        }
    }
}

fn cancellation_requested(cancel_requested: &AtomicBool) -> bool {
    cancel_requested.load(Ordering::SeqCst)
}

#[cfg(unix)]
fn configure_cargo_process_for_tree_cancellation(process: &mut Command) {
    use std::os::unix::process::CommandExt;

    process.process_group(0);
}

#[cfg(not(unix))]
fn configure_cargo_process_for_tree_cancellation(_process: &mut Command) {}

fn terminate_cargo_process_tree(child: &mut Child, label: &str) -> String {
    let child_id = child.id();
    let mut diagnostics = Vec::new();
    if let Some(result) = terminate_platform_process_tree(child_id, label) {
        diagnostics.push(result.diagnostic);
        if result.succeeded {
            return diagnostics.join("\n");
        }
    }
    diagnostics.push(match child.kill() {
        Ok(()) => format!("{label} cancelled; Cargo process was terminated"),
        Err(error) => {
            format!("{label} cancellation requested but Cargo termination failed: {error}")
        }
    });
    diagnostics.join("\n")
}

struct PlatformProcessTreeTermination {
    diagnostic: String,
    succeeded: bool,
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
            diagnostic: format!("{label} cancelled; Cargo process tree was terminated"),
            succeeded: true,
        },
        Ok(output) => PlatformProcessTreeTermination {
            diagnostic: format!(
                "{label} cancellation requested but taskkill failed with status {:?}: {}{}",
                output.status.code(),
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            ),
            succeeded: false,
        },
        Err(error) => PlatformProcessTreeTermination {
            diagnostic: format!("{label} cancellation requested but taskkill failed: {error}"),
            succeeded: false,
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
            diagnostic: format!("{label} cancelled; Cargo process group was terminated"),
            succeeded: true,
        },
        Ok(output) => PlatformProcessTreeTermination {
            diagnostic: format!(
                "{label} cancellation requested but process-group kill failed with status {:?}: {}{}",
                output.status.code(),
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            ),
            succeeded: false,
        },
        Err(error) => PlatformProcessTreeTermination {
            diagnostic: format!(
                "{label} cancellation requested but process-group kill failed: {error}"
            ),
            succeeded: false,
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

fn cancelled_invocation(command: Vec<String>, stderr: String) -> EditorExportCargoInvocation {
    EditorExportCargoInvocation {
        command,
        status_code: None,
        success: false,
        stdout: String::new(),
        stderr,
    }
}

fn read_pipe_to_string<T>(mut pipe: T) -> JoinHandle<String>
where
    T: Read + Send + 'static,
{
    thread::spawn(move || {
        let mut bytes = Vec::new();
        let _ = pipe.read_to_end(&mut bytes);
        String::from_utf8_lossy(&bytes).to_string()
    })
}

fn collect_reader_output(
    reader: Option<JoinHandle<String>>,
    label: &str,
    stream_name: &str,
) -> Result<String, String> {
    reader
        .map(|reader| {
            reader
                .join()
                .map_err(|_| format!("failed to collect Cargo {stream_name} for {label}"))
        })
        .unwrap_or_else(|| Ok(String::new()))
}

fn invocation_from_status(
    command: Vec<String>,
    status: ExitStatus,
    stdout: String,
    stderr: String,
) -> EditorExportCargoInvocation {
    EditorExportCargoInvocation {
        command,
        status_code: status.code(),
        success: status.success(),
        stdout,
        stderr,
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::AtomicBool;

    use super::*;

    #[test]
    fn cargo_process_returns_cancelled_invocation_before_spawn() {
        let cancel_requested = AtomicBool::new(true);
        let invocation = invoke_cargo_process(
            "cargo".to_string(),
            vec!["build".to_string()],
            None,
            Some(&cancel_requested),
            "test export build",
        )
        .expect("pre-cancelled cargo process should return a diagnostic invocation");

        assert_eq!(invocation.command, vec!["cargo", "build"]);
        assert_eq!(invocation.status_code, None);
        assert!(!invocation.success);
        assert!(invocation
            .stderr
            .contains("test export build cancelled before Cargo started"));
    }

    #[cfg(windows)]
    #[test]
    fn cargo_process_tree_termination_args_use_windows_tree_kill() {
        assert_eq!(
            platform_process_tree_termination_args(42),
            vec!["/PID", "42", "/T", "/F"]
        );
    }

    #[cfg(all(unix, not(windows)))]
    #[test]
    fn cargo_process_tree_termination_args_use_unix_group_kill() {
        assert_eq!(
            platform_process_tree_termination_args(42),
            vec!["-KILL", "-42"]
        );
    }
}

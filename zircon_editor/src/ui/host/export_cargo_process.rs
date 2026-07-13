use std::path::Path;
use std::process::{Child, Command, ExitStatus, Stdio};
use std::thread;
use std::time::Duration;

use crate::core::jobs::{CancellationToken, EditorJobSystem};

use super::editor_manager_plugins_export::EditorExportCargoInvocation;
use super::export_process_support::{
    configure_process_tree_cancellation, create_output_capture, final_output_drain,
    join_output_with_poll, terminate_process_tree, ExportProcessChildGuard, ExportProcessError,
    ExportProcessJoin,
};

const CARGO_PROCESS_CANCEL_POLL_INTERVAL: Duration = Duration::from_millis(100);

pub(in crate::ui::host) fn invoke_cargo_process(
    jobs: &EditorJobSystem,
    cargo: String,
    args: Vec<String>,
    current_dir: Option<&Path>,
    cancel: &CancellationToken,
    label: &str,
) -> Result<EditorExportCargoInvocation, ExportProcessError> {
    invoke_cargo_process_with_join(jobs, cargo, args, current_dir, cancel, label)
}

fn invoke_cargo_process_with_join<J: ExportProcessJoin>(
    jobs: &J,
    cargo: String,
    args: Vec<String>,
    current_dir: Option<&Path>,
    cancel: &CancellationToken,
    label: &str,
) -> Result<EditorExportCargoInvocation, ExportProcessError> {
    let mut command = Vec::with_capacity(args.len() + 1);
    command.push(cargo.clone());
    command.extend(args.clone());

    if cancel.is_cancelled() {
        return Ok(cancelled_invocation(
            command,
            format!("{label} cancelled before Cargo started"),
        ));
    }

    let (stdout_writer, stderr_writer, mut output_readers) = create_output_capture(label)?;
    let mut process = Command::new(&cargo);
    process
        .args(&args)
        .stdout(Stdio::from(stdout_writer))
        .stderr(Stdio::from(stderr_writer));
    if let Some(current_dir) = current_dir {
        process.current_dir(current_dir);
    }
    configure_process_tree_cancellation(&mut process);

    let child = process.spawn().map_err(|error| {
        ExportProcessError::io("failed to invoke Cargo", label, None, None, error)
    })?;
    let mut child_guard = ExportProcessChildGuard::new(child, label);
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let outcome = loop {
        let child = child_guard.child_mut();
        let (output, polled) = join_output_with_poll(jobs, &mut output_readers, || {
            poll_cargo_process(child, cancel, label)
        });
        let output = match output {
            Ok(output) => output,
            Err(error) => {
                let termination = terminate_process_tree(child_guard.child_mut(), label);
                if termination.succeeded {
                    let _ = child_guard.child_mut().wait();
                }
                return Err(error.with_cleanup(termination.diagnostic, termination.error));
            }
        };
        append_captured_output(output, &mut stdout, &mut stderr);
        let polled = match polled {
            Ok(polled) => polled,
            Err(error) => {
                let termination = terminate_process_tree(child_guard.child_mut(), label);
                if termination.succeeded {
                    let _ = child_guard.child_mut().wait();
                }
                return Err(error.with_cleanup(termination.diagnostic, termination.error));
            }
        };
        if let Some(outcome) = polled {
            break outcome;
        }
        thread::sleep(CARGO_PROCESS_CANCEL_POLL_INTERVAL);
    };
    append_captured_output(
        final_output_drain(jobs, &mut output_readers)?,
        &mut stdout,
        &mut stderr,
    );
    let stdout = String::from_utf8_lossy(&stdout).to_string();
    let stderr = String::from_utf8_lossy(&stderr).to_string();
    child_guard.disarm();
    match outcome {
        CargoProcessOutcome::Cancelled {
            status,
            kill_diagnostic,
        } => {
            let mut invocation = invocation_from_status(command, status, stdout, stderr);
            invocation.success = false;
            if invocation.stderr.is_empty() {
                invocation.stderr = kill_diagnostic;
            } else {
                invocation.stderr.push('\n');
                invocation.stderr.push_str(&kill_diagnostic);
            }
            Ok(invocation)
        }
        CargoProcessOutcome::Completed(status) => {
            Ok(invocation_from_status(command, status, stdout, stderr))
        }
    }
}

enum CargoProcessOutcome {
    Completed(ExitStatus),
    Cancelled {
        status: ExitStatus,
        kill_diagnostic: String,
    },
}

fn poll_cargo_process(
    child: &mut Child,
    cancel: &CancellationToken,
    label: &str,
) -> Result<Option<CargoProcessOutcome>, ExportProcessError> {
    if cancel.is_cancelled() {
        let termination = terminate_process_tree(child, label);
        if !termination.succeeded {
            if let Some(status) = child.try_wait().map_err(|error| {
                ExportProcessError::io(
                    "failed to poll cancelled Cargo process",
                    label,
                    None,
                    None,
                    error,
                )
            })? {
                return Ok(Some(CargoProcessOutcome::Cancelled {
                    status,
                    kill_diagnostic: termination.diagnostic,
                }));
            }
            return Err(ExportProcessError::TerminationFailed {
                label: label.to_string(),
                diagnostic: termination.diagnostic,
                source: Box::new(
                    termination
                        .error
                        .expect("failed process-tree termination must retain a typed cause"),
                ),
            });
        }
        let status = child.wait().map_err(|error| {
            ExportProcessError::io(
                "failed to collect cancelled Cargo output",
                label,
                None,
                None,
                error,
            )
        })?;
        return Ok(Some(CargoProcessOutcome::Cancelled {
            status,
            kill_diagnostic: termination.diagnostic,
        }));
    }
    child
        .try_wait()
        .map(|status| status.map(CargoProcessOutcome::Completed))
        .map_err(|error| {
            ExportProcessError::io("failed to poll Cargo process", label, None, None, error)
        })
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

fn append_captured_output(
    output: super::export_process_support::CapturedOutputChunk,
    stdout: &mut Vec<u8>,
    stderr: &mut Vec<u8>,
) {
    stdout.extend(output.stdout);
    stderr.extend(output.stderr);
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
    use zircon_runtime::core::runtime::tasks::{TaskPool, TaskPoolDescriptor};

    use super::*;

    #[test]
    fn cargo_process_returns_cancelled_invocation_before_spawn() {
        let jobs = crate::core::jobs::test_job_system();
        let cancel_requested = crate::core::jobs::CancellationToken::default();
        cancel_requested.cancel();
        let invocation = invoke_cargo_process(
            &jobs,
            "cargo".to_string(),
            vec!["build".to_string()],
            None,
            &cancel_requested,
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

    #[test]
    fn cargo_capture_and_poll_complete_on_a_single_runtime_worker() {
        let pool = TaskPool::new(TaskPoolDescriptor::compute().with_worker_threads(1));
        let cancel = CancellationToken::default();
        #[cfg(windows)]
        let (program, args) = (
            "cmd".to_string(),
            vec![
                "/C".to_string(),
                "(for /L %i in (1,1,5000) do @echo stdout-line-%i) & (for /L %i in (1,1,5000) do @echo stderr-line-%i 1>&2)".to_string(),
            ],
        );
        #[cfg(unix)]
        let (program, args) = (
            "sh".to_string(),
            vec![
                "-c".to_string(),
                "i=1; while [ $i -le 5000 ]; do printf 'stdout-line-%s\\n' \"$i\"; printf 'stderr-line-%s\\n' \"$i\" >&2; i=$((i+1)); done".to_string(),
            ],
        );

        let invocation = invoke_cargo_process_with_join(
            &pool,
            program,
            args,
            None,
            &cancel,
            "single-worker export process",
        )
        .expect("single-worker export process should complete");

        assert!(invocation.success);
        assert!(invocation.stdout.contains("stdout-line-5000"));
        assert!(invocation.stderr.contains("stderr-line-5000"));
    }
}

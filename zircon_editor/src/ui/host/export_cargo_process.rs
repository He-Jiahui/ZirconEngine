use std::collections::VecDeque;
use std::path::Path;
use std::process::{Child, Command, ExitStatus, Stdio};
use std::thread;
use std::time::Duration;

use crate::core::jobs::{CancellationToken, EditorJobSystem};

use super::editor_manager_plugins_export::EditorExportCargoInvocation;
use super::export_process_support::{
    configure_process_tree_cancellation, create_output_capture, join_output_with_poll,
    terminate_process_tree, ExportProcessChildGuard, ExportProcessError, ExportProcessJoin,
    ExportProcessOutputReaders,
};

const CARGO_PROCESS_CANCEL_POLL_INTERVAL: Duration = Duration::from_millis(100);
const MAX_CARGO_OUTPUT_TAIL_BYTES: usize = 256 * 1024;

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
    let mut stdout = BoundedOutputTail::new(MAX_CARGO_OUTPUT_TAIL_BYTES);
    let mut stderr = BoundedOutputTail::new(MAX_CARGO_OUTPUT_TAIL_BYTES);
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
    drain_captured_output(jobs, &mut output_readers, &mut stdout, &mut stderr)?;
    let stdout = stdout.finish();
    let stderr = stderr.finish();
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

#[derive(Debug)]
struct BoundedOutputTail {
    bytes: VecDeque<u8>,
    max_bytes: usize,
    total_bytes: usize,
    discarded_bytes: usize,
}

impl BoundedOutputTail {
    fn new(max_bytes: usize) -> Self {
        assert!(max_bytes > 0, "output tail budget must be positive");
        Self {
            bytes: VecDeque::with_capacity(max_bytes),
            max_bytes,
            total_bytes: 0,
            discarded_bytes: 0,
        }
    }

    fn append(&mut self, bytes: &[u8]) {
        self.total_bytes = self.total_bytes.saturating_add(bytes.len());
        if bytes.len() >= self.max_bytes {
            let newly_discarded = self
                .bytes
                .len()
                .saturating_add(bytes.len().saturating_sub(self.max_bytes));
            self.discarded_bytes = self.discarded_bytes.saturating_add(newly_discarded);
            self.bytes.clear();
            self.bytes
                .extend(bytes[bytes.len() - self.max_bytes..].iter().copied());
            return;
        }

        let overflow = self
            .bytes
            .len()
            .saturating_add(bytes.len())
            .saturating_sub(self.max_bytes);
        if overflow > 0 {
            self.bytes.drain(..overflow);
            self.discarded_bytes = self.discarded_bytes.saturating_add(overflow);
        }
        self.bytes.extend(bytes.iter().copied());
    }

    fn finish(self) -> String {
        let mut output = String::with_capacity(self.bytes.len() + 96);
        if self.discarded_bytes > 0 {
            output.push_str(&format!(
                "[output truncated: retained last {} bytes of {} total bytes; discarded {} bytes]\n",
                self.bytes.len(), self.total_bytes, self.discarded_bytes
            ));
        }
        let bytes = self.bytes.into_iter().collect::<Vec<_>>();
        output.push_str(&String::from_utf8_lossy(&bytes));
        output
    }
}

fn append_captured_output(
    output: super::export_process_support::CapturedOutputChunk,
    stdout: &mut BoundedOutputTail,
    stderr: &mut BoundedOutputTail,
) {
    stdout.append(&output.stdout);
    stderr.append(&output.stderr);
}

fn drain_captured_output<J: ExportProcessJoin>(
    jobs: &J,
    readers: &mut ExportProcessOutputReaders,
    stdout: &mut BoundedOutputTail,
    stderr: &mut BoundedOutputTail,
) -> Result<(), ExportProcessError> {
    loop {
        let (output, ()) = join_output_with_poll(jobs, readers, || ());
        let output = output?;
        let complete = output.stdout.is_empty() && output.stderr.is_empty();
        append_captured_output(output, stdout, stderr);
        if complete {
            return Ok(());
        }
    }
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
                "(for /L %i in (1,1,20000) do @echo stdout-line-%i) & (for /L %i in (1,1,20000) do @echo stderr-line-%i 1>&2)".to_string(),
            ],
        );
        #[cfg(unix)]
        let (program, args) = (
            "sh".to_string(),
            vec![
                "-c".to_string(),
                "i=1; while [ $i -le 20000 ]; do printf 'stdout-line-%s\\n' \"$i\"; printf 'stderr-line-%s\\n' \"$i\" >&2; i=$((i+1)); done".to_string(),
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
        assert!(invocation.stdout.contains("stdout-line-20000"));
        assert!(invocation.stderr.contains("stderr-line-20000"));
        assert!(invocation.stdout.starts_with("[output truncated:"));
        assert!(invocation.stderr.starts_with("[output truncated:"));
        assert!(invocation.stdout.len() <= MAX_CARGO_OUTPUT_TAIL_BYTES + 128);
        assert!(invocation.stderr.len() <= MAX_CARGO_OUTPUT_TAIL_BYTES + 128);
    }

    #[test]
    fn cargo_output_tail_keeps_a_constant_memory_budget_and_reports_discarded_bytes() {
        let mut tail = BoundedOutputTail::new(8);
        tail.append(b"0123456789abcdef");

        let output = tail.finish();

        assert!(output.starts_with(
            "[output truncated: retained last 8 bytes of 16 total bytes; discarded 8 bytes]\n"
        ));
        assert!(output.ends_with("89abcdef"));
        assert!(output.len() < 128);
    }

    #[test]
    fn cargo_output_tail_preserves_the_exact_tail_across_capture_chunks() {
        let mut tail = BoundedOutputTail::new(8);
        tail.append(b"0123");
        tail.append(b"456789ab");

        let output = tail.finish();

        assert!(output.starts_with(
            "[output truncated: retained last 8 bytes of 12 total bytes; discarded 4 bytes]\n"
        ));
        assert!(output.ends_with("456789ab"));
    }

    #[test]
    fn export_process_final_drains_do_not_reaggregate_the_complete_stream() {
        let cargo_source = include_str!("export_cargo_process.rs");
        let support_source = include_str!("export_process_support/output_capture.rs");
        let wizard_source =
            include_str!("editor_manager_plugins_export/export_build/wizard/execution.rs");

        assert!(!cargo_source.contains(concat!("final_output_", "drain")));
        assert!(!support_source.contains(concat!("final_output_", "drain")));
        assert!(!wizard_source.contains(concat!("final_output_", "drain")));
        assert!(cargo_source.contains("fn drain_captured_output"));
        assert!(wizard_source.contains("let complete = output.stdout.is_empty()"));
        assert!(wizard_source.contains(".record(output, complete, emit_output)"));
    }
}

use std::{
    io::{BufRead, BufReader, Read},
    process::{Child, Command, ExitStatus, Stdio},
    sync::mpsc::{channel, RecvTimeoutError, Sender},
    thread::{self, JoinHandle},
    time::Duration,
};

use zircon_runtime::plugin::ExportPipelineStage;

use super::{
    ExportStageProgressKind, ExportWizardPipelinePlan, ExportWizardPipelineStageCommand,
    ExportWizardProgressState,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExportWizardCommandExecution {
    pub exit_code: Option<i32>,
    pub stdout_lines: Vec<String>,
    pub stderr_lines: Vec<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExportWizardCommandOutputStream {
    Stdout,
    Stderr,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExportWizardCommandOutputLine {
    pub stream: ExportWizardCommandOutputStream,
    pub line: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExportWizardStageExecution {
    pub stage: ExportPipelineStage,
    pub command: Vec<String>,
    pub exit_code: Option<i32>,
    pub stdout_lines: Vec<String>,
    pub stderr_lines: Vec<String>,
    pub diagnostics: Vec<String>,
    pub cancelled: bool,
    pub fatal: bool,
    pub progress: ExportWizardProgressState,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExportWizardPipelineExecution {
    pub stages: Vec<ExportWizardStageExecution>,
    pub progress: ExportWizardProgressState,
    pub diagnostics: Vec<String>,
    pub fatal: bool,
}

pub trait ExportWizardCommandRunner {
    fn run(
        &mut self,
        command: &ExportWizardPipelineStageCommand,
    ) -> Result<ExportWizardCommandExecution, String>;

    fn run_with_output(
        &mut self,
        command: &ExportWizardPipelineStageCommand,
        emit_output: &mut dyn FnMut(ExportWizardCommandOutputLine),
    ) -> Result<ExportWizardCommandExecution, String> {
        let execution = self.run(command)?;
        for line in &execution.stdout_lines {
            emit_output(ExportWizardCommandOutputLine {
                stream: ExportWizardCommandOutputStream::Stdout,
                line: line.clone(),
            });
        }
        for line in &execution.stderr_lines {
            emit_output(ExportWizardCommandOutputLine {
                stream: ExportWizardCommandOutputStream::Stderr,
                line: line.clone(),
            });
        }
        Ok(execution)
    }

    fn run_with_output_and_cancel(
        &mut self,
        command: &ExportWizardPipelineStageCommand,
        emit_output: &mut dyn FnMut(ExportWizardCommandOutputLine),
        should_cancel: &mut dyn FnMut() -> bool,
    ) -> Result<ExportWizardCommandExecution, String> {
        let _ = should_cancel;
        self.run_with_output(command, emit_output)
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct ProcessCommandRunner;

impl ExportWizardCommandRunner for ProcessCommandRunner {
    fn run(
        &mut self,
        command: &ExportWizardPipelineStageCommand,
    ) -> Result<ExportWizardCommandExecution, String> {
        self.run_with_output_and_cancel(command, &mut |_| {}, &mut || false)
    }

    fn run_with_output(
        &mut self,
        command: &ExportWizardPipelineStageCommand,
        emit_output: &mut dyn FnMut(ExportWizardCommandOutputLine),
    ) -> Result<ExportWizardCommandExecution, String> {
        self.run_with_output_and_cancel(command, emit_output, &mut || false)
    }

    fn run_with_output_and_cancel(
        &mut self,
        command: &ExportWizardPipelineStageCommand,
        emit_output: &mut dyn FnMut(ExportWizardCommandOutputLine),
        should_cancel: &mut dyn FnMut() -> bool,
    ) -> Result<ExportWizardCommandExecution, String> {
        if should_cancel() {
            return Err(format!(
                "export stage {:?} was cancelled before process launch",
                command.stage
            ));
        }

        let mut process = Command::new(&command.program);
        process.args(&command.args);
        if let Some(working_dir) = command.working_dir.as_deref() {
            process.current_dir(working_dir);
        }
        process.stdout(Stdio::piped()).stderr(Stdio::piped());
        let mut child = process.spawn().map_err(|error| {
            let working_dir = command.working_dir.as_deref().unwrap_or("<current>");
            format!(
                "failed to execute export stage {:?} in {working_dir}: {error}",
                command.stage
            )
        })?;

        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| format!("failed to capture {:?} stdout", command.stage))?;
        let stderr = child
            .stderr
            .take()
            .ok_or_else(|| format!("failed to capture {:?} stderr", command.stage))?;

        let (sender, receiver) = channel();
        let stdout_reader = spawn_output_reader(
            ExportWizardCommandOutputStream::Stdout,
            stdout,
            sender.clone(),
        );
        let stderr_reader =
            spawn_output_reader(ExportWizardCommandOutputStream::Stderr, stderr, sender);

        let mut stdout_lines = Vec::new();
        let mut stderr_lines = Vec::new();
        let mut read_errors = Vec::new();
        let mut output_closed = false;
        let mut process_status = None;
        let poll_interval = Duration::from_millis(25);
        while !output_closed || process_status.is_none() {
            if output_closed {
                thread::sleep(poll_interval);
            } else {
                match receiver.recv_timeout(poll_interval) {
                    Ok(message) => record_output_message(
                        message,
                        &mut stdout_lines,
                        &mut stderr_lines,
                        &mut read_errors,
                        emit_output,
                    ),
                    Err(RecvTimeoutError::Timeout) => {}
                    Err(RecvTimeoutError::Disconnected) => output_closed = true,
                }
            }

            if process_status.is_none() {
                if should_cancel() {
                    process_status = Some(terminate_child_for_cancel(&mut child, command.stage)?);
                } else if let Some(status) = try_wait_export_process(&mut child, command.stage)? {
                    process_status = Some(status);
                }
            }
        }

        join_output_reader(stdout_reader, command.stage, "stdout")?;
        join_output_reader(stderr_reader, command.stage, "stderr")?;
        let status = match process_status {
            Some(status) => status,
            None => child.wait().map_err(|error| {
                format!(
                    "failed to wait for export stage {:?} process: {error}",
                    command.stage
                )
            })?,
        };
        if let Some(error) = read_errors.into_iter().next() {
            return Err(error);
        }

        Ok(ExportWizardCommandExecution {
            exit_code: status.code(),
            stdout_lines,
            stderr_lines,
        })
    }
}

pub fn execute_export_wizard_stage(
    command: &ExportWizardPipelineStageCommand,
    runner: &mut impl ExportWizardCommandRunner,
    progress: &mut ExportWizardProgressState,
) -> ExportWizardStageExecution {
    execute_export_wizard_stage_with_output(command, runner, progress, &mut |_, _| {})
}

pub fn execute_export_wizard_stage_with_output(
    command: &ExportWizardPipelineStageCommand,
    runner: &mut impl ExportWizardCommandRunner,
    progress: &mut ExportWizardProgressState,
    emit_output: &mut impl FnMut(ExportWizardCommandOutputLine, &ExportWizardProgressState),
) -> ExportWizardStageExecution {
    execute_export_wizard_stage_with_output_and_cancel(
        command,
        runner,
        progress,
        emit_output,
        &mut || false,
    )
}

pub fn execute_export_wizard_stage_with_output_and_cancel(
    command: &ExportWizardPipelineStageCommand,
    runner: &mut impl ExportWizardCommandRunner,
    progress: &mut ExportWizardProgressState,
    emit_output: &mut impl FnMut(ExportWizardCommandOutputLine, &ExportWizardProgressState),
    should_cancel: &mut impl FnMut() -> bool,
) -> ExportWizardStageExecution {
    let mut diagnostics = Vec::new();
    let mut fatal = false;
    for missing_input in &command.missing_inputs {
        fatal = true;
        diagnostics.push(format!(
            "export stage {:?} is missing required input {missing_input}",
            command.stage
        ));
    }
    let argv = command.argv();
    if !command.missing_inputs.is_empty() {
        return ExportWizardStageExecution {
            stage: command.stage,
            command: argv,
            exit_code: None,
            stdout_lines: Vec::new(),
            stderr_lines: Vec::new(),
            diagnostics,
            cancelled: false,
            fatal,
            progress: progress.clone(),
        };
    }

    if should_cancel() {
        diagnostics.push(format!(
            "export stage {:?} was cancelled before process execution",
            command.stage
        ));
        return ExportWizardStageExecution {
            stage: command.stage,
            command: argv,
            exit_code: None,
            stdout_lines: Vec::new(),
            stderr_lines: Vec::new(),
            diagnostics,
            cancelled: true,
            fatal: false,
            progress: progress.clone(),
        };
    }

    let mut observe_output = |output: ExportWizardCommandOutputLine| {
        if output.stream == ExportWizardCommandOutputStream::Stdout {
            progress.push_stdout_line(&output.line);
        }
        emit_output(output, progress);
    };

    match runner.run_with_output_and_cancel(command, &mut observe_output, should_cancel) {
        Ok(execution) => {
            let cancelled = should_cancel();
            if cancelled {
                diagnostics.push(format!(
                    "export stage {:?} was cancelled during process execution",
                    command.stage
                ));
            } else {
                diagnostics.extend(progress_stage_diagnostics(progress, command.stage));
                for line in &execution.stderr_lines {
                    diagnostics.push(format!("{:?} stderr: {line}", command.stage));
                }
                if execution.exit_code != Some(0) {
                    fatal = true;
                    diagnostics.push(format!(
                        "export stage {:?} exited with code {}",
                        command.stage,
                        execution
                            .exit_code
                            .map(|code| code.to_string())
                            .unwrap_or_else(|| "unknown".to_string())
                    ));
                }
                if progress
                    .snapshot(command.stage)
                    .is_some_and(|snapshot| snapshot.kind == ExportStageProgressKind::Fatal)
                {
                    fatal = true;
                    diagnostics.push(format!(
                        "export stage {:?} reported fatal status",
                        command.stage
                    ));
                }
            }
            ExportWizardStageExecution {
                stage: command.stage,
                command: argv,
                exit_code: execution.exit_code,
                stdout_lines: execution.stdout_lines,
                stderr_lines: execution.stderr_lines,
                diagnostics,
                cancelled,
                fatal,
                progress: progress.clone(),
            }
        }
        Err(error) => {
            let cancelled = should_cancel();
            fatal = !cancelled;
            diagnostics.push(error);
            ExportWizardStageExecution {
                stage: command.stage,
                command: argv,
                exit_code: None,
                stdout_lines: Vec::new(),
                stderr_lines: Vec::new(),
                diagnostics,
                cancelled,
                fatal,
                progress: progress.clone(),
            }
        }
    }
}

fn progress_stage_diagnostics(
    progress: &ExportWizardProgressState,
    stage: ExportPipelineStage,
) -> Vec<String> {
    progress
        .snapshot(stage)
        .map(|snapshot| snapshot.diagnostics.clone())
        .unwrap_or_default()
}

pub fn execute_export_wizard_pipeline(
    plan: &ExportWizardPipelinePlan,
    runner: &mut impl ExportWizardCommandRunner,
) -> ExportWizardPipelineExecution {
    let mut progress =
        ExportWizardProgressState::for_stages(plan.stages.iter().map(|command| command.stage));
    let mut stages = Vec::new();
    let mut diagnostics = plan.diagnostics.clone();
    let mut fatal = !plan.diagnostics.is_empty();
    if fatal {
        return ExportWizardPipelineExecution {
            stages,
            progress,
            diagnostics,
            fatal,
        };
    }

    for command in &plan.stages {
        let stage_execution = execute_export_wizard_stage(command, runner, &mut progress);
        diagnostics.extend(stage_execution.diagnostics.iter().cloned());
        let should_stop = stage_execution.fatal;
        stages.push(stage_execution);
        if should_stop {
            fatal = true;
            break;
        }
    }

    ExportWizardPipelineExecution {
        stages,
        progress,
        diagnostics,
        fatal,
    }
}

fn record_output_message(
    message: Result<ExportWizardCommandOutputLine, String>,
    stdout_lines: &mut Vec<String>,
    stderr_lines: &mut Vec<String>,
    read_errors: &mut Vec<String>,
    emit_output: &mut dyn FnMut(ExportWizardCommandOutputLine),
) {
    match message {
        Ok(output) => {
            match output.stream {
                ExportWizardCommandOutputStream::Stdout => {
                    stdout_lines.push(output.line.clone());
                }
                ExportWizardCommandOutputStream::Stderr => {
                    stderr_lines.push(output.line.clone());
                }
            }
            emit_output(output);
        }
        Err(error) => read_errors.push(error),
    }
}

fn try_wait_export_process(
    child: &mut Child,
    stage: ExportPipelineStage,
) -> Result<Option<ExitStatus>, String> {
    child
        .try_wait()
        .map_err(|error| format!("failed to poll export stage {stage:?} process: {error}"))
}

fn terminate_child_for_cancel(
    child: &mut Child,
    stage: ExportPipelineStage,
) -> Result<ExitStatus, String> {
    if let Some(status) = try_wait_export_process(child, stage)? {
        return Ok(status);
    }

    match child.kill() {
        Ok(()) => child.wait().map_err(|error| {
            format!("failed to wait for cancelled export stage {stage:?}: {error}")
        }),
        Err(error) => {
            if let Some(status) = try_wait_export_process(child, stage)? {
                Ok(status)
            } else {
                Err(format!(
                    "failed to terminate cancelled export stage {stage:?} process: {error}"
                ))
            }
        }
    }
}

fn spawn_output_reader<R>(
    stream: ExportWizardCommandOutputStream,
    reader: R,
    sender: Sender<Result<ExportWizardCommandOutputLine, String>>,
) -> JoinHandle<()>
where
    R: Read + Send + 'static,
{
    thread::spawn(move || {
        for line in BufReader::new(reader).lines() {
            let message = line
                .map(|line| ExportWizardCommandOutputLine { stream, line })
                .map_err(|error| format!("failed to read export process {stream:?}: {error}"));
            if sender.send(message).is_err() {
                break;
            }
        }
    })
}

fn join_output_reader(
    handle: JoinHandle<()>,
    stage: ExportPipelineStage,
    stream_name: &str,
) -> Result<(), String> {
    handle.join().map_err(|_| {
        format!("export stage {stage:?} {stream_name} reader panicked while streaming output")
    })
}

use std::{
    fmt,
    process::{Child, Command, ExitStatus, Stdio},
    thread,
    time::Duration,
};

use crate::core::jobs::{EditorJobSystem, JobFailure};
use crate::ui::host::export_process_support::{
    configure_process_tree_cancellation, create_output_capture, final_output_drain,
    join_output_with_poll, terminate_process_tree, ExportProcessChildGuard, ExportProcessError,
};
use zircon_runtime_interface::export::ExportStage;

use super::super::EditorExportBuildError;
use super::{
    ExportStageProgressKind, ExportWizardCoreStageProjection, ExportWizardPipelinePlan,
    ExportWizardPipelineStageCommand, ExportWizardProgressState,
};

mod core_pipeline;
mod output_capture;

use super::output_tail::{push_bounded_output_line, retain_bounded_output_tail};
use core_pipeline::{run_core_compile_host, run_core_platform_bundle};
use output_capture::ExportWizardOutputCapture;

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
    pub stage: ExportStage,
    pub command: Vec<String>,
    pub exit_code: Option<i32>,
    pub stdout_lines: Vec<String>,
    pub stderr_lines: Vec<String>,
    pub diagnostics: Vec<String>,
    pub failure: Option<JobFailure>,
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
    ) -> Result<ExportWizardCommandExecution, EditorExportBuildError>;

    fn run_with_output(
        &mut self,
        command: &ExportWizardPipelineStageCommand,
        emit_output: &mut (dyn FnMut(ExportWizardCommandOutputLine) + Send),
    ) -> Result<ExportWizardCommandExecution, EditorExportBuildError> {
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
        emit_output: &mut (dyn FnMut(ExportWizardCommandOutputLine) + Send),
        should_cancel: &mut (dyn FnMut() -> bool + Send),
    ) -> Result<ExportWizardCommandExecution, EditorExportBuildError> {
        let _ = should_cancel;
        self.run_with_output(command, emit_output)
    }
}

#[derive(Clone)]
pub struct ProcessCommandRunner {
    jobs: EditorJobSystem,
}

impl fmt::Debug for ProcessCommandRunner {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ProcessCommandRunner")
            .finish_non_exhaustive()
    }
}

impl ProcessCommandRunner {
    pub fn new(jobs: EditorJobSystem) -> Self {
        Self { jobs }
    }
}

impl ExportWizardCommandRunner for ProcessCommandRunner {
    fn run(
        &mut self,
        command: &ExportWizardPipelineStageCommand,
    ) -> Result<ExportWizardCommandExecution, EditorExportBuildError> {
        self.run_with_output_and_cancel(command, &mut |_| {}, &mut || false)
    }

    fn run_with_output(
        &mut self,
        command: &ExportWizardPipelineStageCommand,
        emit_output: &mut (dyn FnMut(ExportWizardCommandOutputLine) + Send),
    ) -> Result<ExportWizardCommandExecution, EditorExportBuildError> {
        self.run_with_output_and_cancel(command, emit_output, &mut || false)
    }

    fn run_with_output_and_cancel(
        &mut self,
        command: &ExportWizardPipelineStageCommand,
        emit_output: &mut (dyn FnMut(ExportWizardCommandOutputLine) + Send),
        should_cancel: &mut (dyn FnMut() -> bool + Send),
    ) -> Result<ExportWizardCommandExecution, EditorExportBuildError> {
        if matches!(
            command.core_projection,
            Some(ExportWizardCoreStageProjection::CompileHost { .. })
        ) {
            let mut execution = run_core_compile_host(self, command, emit_output, should_cancel)?;
            project_core_stage_success(command, &mut execution, emit_output)?;
            return Ok(execution);
        }
        if let Some(ExportWizardCoreStageProjection::PlatformBundle { dry_run, .. }) =
            &command.core_projection
        {
            if !dry_run {
                run_core_platform_bundle(command)?;
            }
        }
        if should_cancel() {
            return Err(EditorExportBuildError::cancelled(format!(
                "{:?} process launch",
                command.stage
            )));
        }

        let label = format!("export stage {:?}", command.stage);
        let mut persisted_output = ExportWizardOutputCapture::open(command)
            .map_err(EditorExportBuildError::materialize)?;
        let (stdout_writer, stderr_writer, mut output_readers) = create_output_capture(&label)?;
        let mut process = Command::new(command.native_program.as_ref().map_or_else(
            || std::ffi::OsStr::new(&command.program),
            |program| program.as_os_str(),
        ));
        if let Some(args) = &command.native_args {
            process.args(args);
        } else {
            process.args(&command.args);
        }
        if let Some(working_dir) = command
            .native_working_dir
            .as_deref()
            .or_else(|| command.working_dir.as_deref().map(std::path::Path::new))
        {
            process.current_dir(working_dir);
        }
        process
            .stdout(Stdio::from(stdout_writer))
            .stderr(Stdio::from(stderr_writer));
        configure_process_tree_cancellation(&mut process);
        let child = process.spawn().map_err(|error| {
            let working_dir = command.working_dir.as_deref().unwrap_or("<current>");
            ExportProcessError::io(
                "failed to execute export stage",
                format!("{:?} in {working_dir}", command.stage),
                None,
                None,
                error,
            )
        })?;
        let mut child_guard = ExportProcessChildGuard::new(child, label.clone());

        let status = loop {
            let child = child_guard.child_mut();
            let (output, polled) = join_output_with_poll(&self.jobs, &mut output_readers, || {
                poll_export_process(child, command.stage, &label, should_cancel)
            });
            let output = match output {
                Ok(output) => output,
                Err(error) => {
                    let termination = terminate_process_tree(child_guard.child_mut(), &label);
                    if termination.succeeded {
                        let _ = child_guard.child_mut().wait();
                    }
                    return Err(error
                        .with_cleanup(termination.diagnostic, termination.error)
                        .into());
                }
            };
            persisted_output
                .record(output, false, emit_output)
                .map_err(EditorExportBuildError::materialize)?;
            let polled = match polled {
                Ok(polled) => polled,
                Err(error) => {
                    let termination = terminate_process_tree(child_guard.child_mut(), &label);
                    if termination.succeeded {
                        let _ = child_guard.child_mut().wait();
                    }
                    return Err(error
                        .with_cleanup(termination.diagnostic, termination.error)
                        .into());
                }
            };
            if let Some(status) = polled {
                break status;
            }
            thread::sleep(Duration::from_millis(25));
        };
        persisted_output
            .record(
                final_output_drain(&self.jobs, &mut output_readers)?,
                true,
                emit_output,
            )
            .map_err(EditorExportBuildError::materialize)?;
        child_guard.disarm();

        let persisted_output = persisted_output
            .finish()
            .map_err(EditorExportBuildError::materialize)?;
        let mut execution = ExportWizardCommandExecution {
            exit_code: status.code(),
            stdout_lines: persisted_output.stdout_lines,
            stderr_lines: persisted_output.stderr_lines,
        };
        for artifact_line in persisted_output.artifact_lines {
            push_bounded_output_line(&mut execution.stdout_lines, artifact_line.line.clone());
            emit_output(artifact_line);
        }
        if status.success() {
            project_core_stage_success(command, &mut execution, emit_output)?;
        }
        Ok(execution)
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
    emit_output: &mut (impl FnMut(ExportWizardCommandOutputLine, &ExportWizardProgressState) + Send),
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
    emit_output: &mut (impl FnMut(ExportWizardCommandOutputLine, &ExportWizardProgressState) + Send),
    should_cancel: &mut (impl FnMut() -> bool + Send),
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
            failure: None,
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
            failure: None,
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
    let mut cancel_observed_during_run = false;
    let mut observe_cancel = || {
        let requested = should_cancel();
        cancel_observed_during_run |= requested;
        requested
    };

    match runner.run_with_output_and_cancel(command, &mut observe_output, &mut observe_cancel) {
        Ok(mut execution) => {
            retain_bounded_output_tail(&mut execution.stdout_lines);
            retain_bounded_output_tail(&mut execution.stderr_lines);
            let cancelled = cancel_observed_during_run;
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
                failure: None,
                cancelled,
                fatal,
                progress: progress.clone(),
            }
        }
        Err(error) => {
            let explicitly_cancelled = matches!(error, EditorExportBuildError::Cancelled { .. });
            let cancelled = explicitly_cancelled;
            fatal = !explicitly_cancelled;
            diagnostics.push(error.to_string());
            let failure = Some(JobFailure::new(error));
            ExportWizardStageExecution {
                stage: command.stage,
                command: argv,
                exit_code: None,
                stdout_lines: Vec::new(),
                stderr_lines: Vec::new(),
                diagnostics,
                failure,
                cancelled,
                fatal,
                progress: progress.clone(),
            }
        }
    }
}

fn progress_stage_diagnostics(
    progress: &ExportWizardProgressState,
    stage: ExportStage,
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
        ExportWizardProgressState::for_stages(plan.ordered_commands().map(|command| command.stage));
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

    for command in plan.ordered_commands() {
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

fn project_core_stage_success(
    command: &ExportWizardPipelineStageCommand,
    execution: &mut ExportWizardCommandExecution,
    emit_output: &mut (dyn FnMut(ExportWizardCommandOutputLine) + Send),
) -> Result<(), EditorExportBuildError> {
    let Some(ExportWizardCoreStageProjection::CompileHost {
        report_path,
        profile,
        host_path,
        build_output_root,
        ..
    }) = &command.core_projection
    else {
        return Ok(());
    };
    let report_path = std::path::Path::new(report_path);
    let parent = report_path.parent().ok_or_else(|| {
        EditorExportBuildError::materialize(std::io::Error::other(
            "CompileHost report path has no parent",
        ))
    })?;
    std::fs::create_dir_all(parent).map_err(EditorExportBuildError::materialize)?;
    let report = serde_json::json!({
        "stage": ExportStage::CompileHost.report_name(),
        "profile": profile,
        "fatal": false,
        "diagnostics": [],
        "host_executable": host_path,
        "staged_engine_root": std::path::Path::new(build_output_root).join("ZirconEngine"),
        "exit_code": execution.exit_code.unwrap_or(0),
        "stdout_lines": execution.stdout_lines.clone(),
        "stderr_lines": execution.stderr_lines.clone(),
        "command": command.argv(),
    });
    let bytes = serde_json::to_vec_pretty(&report)
        .map_err(|source| EditorExportBuildError::materialize(std::io::Error::other(source)))?;
    std::fs::write(report_path, bytes).map_err(EditorExportBuildError::materialize)?;
    for line in [
        format!("report={}", report_path.display()),
        format!("host={host_path}"),
    ] {
        push_bounded_output_line(&mut execution.stdout_lines, line.clone());
        emit_output(ExportWizardCommandOutputLine {
            stream: ExportWizardCommandOutputStream::Stdout,
            line,
        });
    }
    Ok(())
}

fn try_wait_export_process(
    child: &mut Child,
    stage: ExportStage,
) -> Result<Option<ExitStatus>, ExportProcessError> {
    child.try_wait().map_err(|error| {
        ExportProcessError::io(
            "failed to poll export stage process",
            format!("{stage:?}"),
            None,
            None,
            error,
        )
    })
}

fn terminate_child_for_cancel(
    child: &mut Child,
    stage: ExportStage,
    label: &str,
) -> Result<ExitStatus, ExportProcessError> {
    if let Some(status) = try_wait_export_process(child, stage)? {
        return Ok(status);
    }

    let termination = terminate_process_tree(child, label);
    if !termination.succeeded {
        if let Some(status) = try_wait_export_process(child, stage)? {
            return Ok(status);
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
    child
        .wait()
        .map_err(|error| {
            ExportProcessError::io(
                "failed to wait for cancelled export stage",
                format!("{stage:?}"),
                None,
                None,
                error,
            )
        })
        .map_err(|error| error.with_cleanup(termination.diagnostic, termination.error))
}

fn poll_export_process(
    child: &mut Child,
    stage: ExportStage,
    label: &str,
    should_cancel: &mut (dyn FnMut() -> bool + Send),
) -> Result<Option<ExitStatus>, ExportProcessError> {
    if should_cancel() {
        return terminate_child_for_cancel(child, stage, label).map(Some);
    }
    try_wait_export_process(child, stage)
}

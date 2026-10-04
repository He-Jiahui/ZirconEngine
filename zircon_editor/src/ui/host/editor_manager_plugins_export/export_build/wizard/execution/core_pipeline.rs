use zircon_runtime_interface::export::ExportStage;

use crate::core::export::{
    CompileHostStage, ExportPipelinePlan, ExportStageNode, ZirconBuildCommand,
    ZirconBuildCommandExecution, ZirconBuildCommandRunner, ZirconBuildStageExecutor,
};

use super::super::{
    EditorExportBuildError, ExportWizardCoreStageProjection, ExportWizardPipelineStageCommand,
};
use super::{
    ExportWizardCommandExecution, ExportWizardCommandOutputLine, ExportWizardCommandOutputStream,
    ExportWizardCommandRunner,
};

pub(super) fn run_core_compile_host(
    process_runner: &mut impl ExportWizardCommandRunner,
    command: &ExportWizardPipelineStageCommand,
    emit_output: &mut (dyn FnMut(ExportWizardCommandOutputLine) + Send),
    should_cancel: &mut (dyn FnMut() -> bool + Send),
) -> Result<ExportWizardCommandExecution, EditorExportBuildError> {
    if should_cancel() {
        return Err(EditorExportBuildError::cancelled("CompileHost admission"));
    }
    let Some(ExportWizardCoreStageProjection::CompileHost {
        preset,
        report_path,
        repo_root,
        build_output_root,
        python,
        cargo,
        locked,
        dry_run,
        ..
    }) = &command.core_projection
    else {
        unreachable!("CompileHost core runner requires CompileHost projection")
    };
    let mut compile_host = CompileHostStage::new(repo_root, build_output_root)
        .with_python(python)
        .with_cargo(cargo);
    if !locked {
        compile_host = compile_host.without_lock();
    }
    if *dry_run {
        compile_host = compile_host.with_dry_run();
    }
    let core_report_path = format!("{report_path}.core.json");
    let adapter = ExportWizardZirconBuildRunner {
        process_runner,
        emit_output,
        should_cancel,
        stage_command: command,
        execution: None,
        report_path: &core_report_path,
    };
    let mut executor = ZirconBuildStageExecutor::new(preset.clone(), compile_host, adapter);
    let plan = ExportPipelinePlan::new([ExportStageNode::new(ExportStage::CompileHost, [])])
        .expect("single-stage CompileHost production graph is valid");
    let resume = load_core_pipeline_report(&core_report_path)?;
    let report = match plan.run(&mut executor, resume.as_ref()) {
        Ok(report) => report,
        Err(error) => {
            let (report, source) = error.into_parts();
            let failure = match source {
                crate::core::export::ZirconBuildStageExecutorError::Build(source) => source,
                crate::core::export::ZirconBuildStageExecutorError::UnsupportedStage { stage } => {
                    EditorExportBuildError::CoreUnsupportedStage { stage }
                }
                crate::core::export::ZirconBuildStageExecutorError::MissingCompileHostRecord => {
                    EditorExportBuildError::CoreMissingCompileHostRecord
                }
                crate::core::export::ZirconBuildStageExecutorError::EncodePreset(source) => {
                    EditorExportBuildError::CorePresetFingerprint { source }
                }
                crate::core::export::ZirconBuildStageExecutorError::Fingerprint(source) => {
                    EditorExportBuildError::CoreArtifactFingerprint { source }
                }
                crate::core::export::ZirconBuildStageExecutorError::BundleLayout(source) => {
                    EditorExportBuildError::PlatformBundleLayout(source)
                }
            };
            return Err(persist_core_failure(&core_report_path, &report, failure));
        }
    };
    write_core_pipeline_report(&core_report_path, &report)?;
    Ok(executor
        .into_runner()
        .execution
        .unwrap_or_else(|| ExportWizardCommandExecution {
            exit_code: Some(0),
            stdout_lines: vec!["core_resume=CompileHost skipped".to_string()],
            stderr_lines: Vec::new(),
        }))
}

pub(super) fn run_core_platform_bundle(
    command: &ExportWizardPipelineStageCommand,
) -> Result<(), EditorExportBuildError> {
    let Some(ExportWizardCoreStageProjection::PlatformBundle {
        preset,
        repo_root,
        build_output_root,
        python,
        cargo,
        locked,
        report_path,
        ..
    }) = &command.core_projection
    else {
        unreachable!("PlatformBundle core runner requires PlatformBundle projection")
    };
    let mut compile_host = CompileHostStage::new(repo_root, build_output_root)
        .with_python(python)
        .with_cargo(cargo);
    if !locked {
        compile_host = compile_host.without_lock();
    }
    let mut executor =
        ZirconBuildStageExecutor::new(preset.clone(), compile_host, RejectUnexpectedCompileHost);
    let core_report_path = format!("{report_path}.core.json");
    let resume = load_core_pipeline_report(&core_report_path)?;
    let report = crate::core::export::zircon_build_stage_plan()
        .run(&mut executor, resume.as_ref())
        .map_err(|error| {
            let (report, source) = error.into_parts();
            let failure = match source {
                crate::core::export::ZirconBuildStageExecutorError::Build(source)
                | crate::core::export::ZirconBuildStageExecutorError::Fingerprint(source) => {
                    EditorExportBuildError::CoreArtifactFingerprint { source }
                }
                crate::core::export::ZirconBuildStageExecutorError::EncodePreset(source) => {
                    EditorExportBuildError::CorePresetFingerprint { source }
                }
                crate::core::export::ZirconBuildStageExecutorError::BundleLayout(source) => {
                    EditorExportBuildError::PlatformBundleLayout(source)
                }
                crate::core::export::ZirconBuildStageExecutorError::UnsupportedStage { stage } => {
                    EditorExportBuildError::CoreUnsupportedStage { stage }
                }
                crate::core::export::ZirconBuildStageExecutorError::MissingCompileHostRecord => {
                    EditorExportBuildError::CoreMissingCompileHostRecord
                }
            };
            persist_core_failure(&core_report_path, &report, failure)
        })?;
    write_core_pipeline_report(&core_report_path, &report)
}

fn persist_core_failure(
    path: &str,
    report: &zircon_runtime_interface::export::ExportPipelineReport,
    failure: EditorExportBuildError,
) -> EditorExportBuildError {
    match write_core_pipeline_report(path, report) {
        Ok(()) => failure,
        Err(persistence) => EditorExportBuildError::CoreFailureReceipt {
            failure: Box::new(failure),
            persistence: Box::new(persistence),
        },
    }
}

struct RejectUnexpectedCompileHost;

impl ZirconBuildCommandRunner for RejectUnexpectedCompileHost {
    type Error = std::io::Error;

    fn run(
        &mut self,
        _command: &ZirconBuildCommand,
    ) -> Result<ZirconBuildCommandExecution, Self::Error> {
        Err(std::io::Error::other(
            "PlatformBundle core pass cannot rebuild CompileHost",
        ))
    }
}

const CORE_PIPELINE_RECEIPT_VERSION: u32 = 1;

#[derive(serde::Serialize, serde::Deserialize)]
struct CorePipelineReceipt<T> {
    #[serde(default)]
    format_version: Option<u32>,
    #[serde(flatten)]
    report: T,
}

fn load_core_pipeline_report(
    path: &str,
) -> Result<Option<zircon_runtime_interface::export::ExportPipelineReport>, EditorExportBuildError>
{
    let path = std::path::Path::new(path);
    let bytes = match std::fs::read(path) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(source) => return Err(EditorExportBuildError::CoreArtifactFingerprint { source }),
    };
    let receipt: CorePipelineReceipt<zircon_runtime_interface::export::ExportPipelineReport> =
        serde_json::from_slice(&bytes).map_err(|source| {
            EditorExportBuildError::CoreArtifactFingerprint {
                source: std::io::Error::new(std::io::ErrorKind::InvalidData, source),
            }
        })?;
    match receipt.format_version {
        Some(CORE_PIPELINE_RECEIPT_VERSION) => Ok(Some(receipt.report)),
        // Unversioned reports may have cached a nonzero exit as Passed.
        None => Ok(None),
        Some(version) => Err(EditorExportBuildError::CoreArtifactFingerprint {
            source: std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("unsupported core export receipt version {version}"),
            ),
        }),
    }
}

fn write_core_pipeline_report(
    path: &str,
    report: &zircon_runtime_interface::export::ExportPipelineReport,
) -> Result<(), EditorExportBuildError> {
    if let Some(parent) = std::path::Path::new(path).parent() {
        std::fs::create_dir_all(parent)
            .map_err(|source| EditorExportBuildError::CoreArtifactFingerprint { source })?;
    }
    let receipt = CorePipelineReceipt {
        format_version: Some(CORE_PIPELINE_RECEIPT_VERSION),
        report,
    };
    let bytes = serde_json::to_vec_pretty(&receipt).map_err(|source| {
        EditorExportBuildError::CoreArtifactFingerprint {
            source: std::io::Error::new(std::io::ErrorKind::InvalidData, source),
        }
    })?;
    let destination = std::path::Path::new(path);
    let staging = destination.with_extension(format!("core.json.{}.staging", std::process::id()));
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&staging)
        .map_err(|source| EditorExportBuildError::CoreArtifactFingerprint { source })?;
    std::io::Write::write_all(&mut file, &bytes)
        .and_then(|_| file.sync_all())
        .map_err(|source| EditorExportBuildError::CoreArtifactFingerprint { source })?;
    drop(file);
    atomic_replace_core_report(&staging, destination)
        .map_err(|source| EditorExportBuildError::CoreArtifactFingerprint { source })
}

#[cfg(windows)]
fn atomic_replace_core_report(
    staging: &std::path::Path,
    destination: &std::path::Path,
) -> std::io::Result<()> {
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Storage::FileSystem::ReplaceFileW;
    if !destination.exists() {
        return std::fs::rename(staging, destination);
    }
    let destination = destination
        .as_os_str()
        .encode_wide()
        .chain(Some(0))
        .collect::<Vec<_>>();
    let staging = staging
        .as_os_str()
        .encode_wide()
        .chain(Some(0))
        .collect::<Vec<_>>();
    let replaced = unsafe {
        ReplaceFileW(
            destination.as_ptr(),
            staging.as_ptr(),
            std::ptr::null(),
            0,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        )
    };
    if replaced == 0 {
        Err(std::io::Error::last_os_error())
    } else {
        Ok(())
    }
}

#[cfg(not(windows))]
fn atomic_replace_core_report(
    staging: &std::path::Path,
    destination: &std::path::Path,
) -> std::io::Result<()> {
    std::fs::rename(staging, destination)
}

struct ExportWizardZirconBuildRunner<'a, R> {
    process_runner: &'a mut R,
    emit_output: &'a mut (dyn FnMut(ExportWizardCommandOutputLine) + Send),
    should_cancel: &'a mut (dyn FnMut() -> bool + Send),
    stage_command: &'a ExportWizardPipelineStageCommand,
    execution: Option<ExportWizardCommandExecution>,
    report_path: &'a str,
}

impl<R: ExportWizardCommandRunner> ZirconBuildCommandRunner
    for ExportWizardZirconBuildRunner<'_, R>
{
    type Error = EditorExportBuildError;

    fn run(
        &mut self,
        command: &ZirconBuildCommand,
    ) -> Result<ZirconBuildCommandExecution, Self::Error> {
        let process_command = ExportWizardPipelineStageCommand {
            stage: ExportStage::CompileHost,
            program: command.program.to_string_lossy().into_owned(),
            working_dir: Some(command.working_directory.display().to_string()),
            args: command
                .args
                .iter()
                .map(|value| value.to_string_lossy().into_owned())
                .collect(),
            consumed_artifacts: self.stage_command.consumed_artifacts.clone(),
            produced_artifacts: self.stage_command.produced_artifacts.clone(),
            expected_stdout_keys: self.stage_command.expected_stdout_keys.clone(),
            missing_inputs: self.stage_command.missing_inputs.clone(),
            core_projection: None,
            native_program: Some(command.program.clone()),
            native_args: Some(command.args.clone()),
            native_working_dir: Some(command.working_directory.clone()),
        };
        // Retire the previous success before a process can mutate its artifacts.
        write_core_pipeline_report(self.report_path, &Default::default())?;
        let mut cancelled = false;
        let mut observe_cancel = || {
            let requested = (self.should_cancel)();
            cancelled |= requested;
            requested
        };
        let execution = self.process_runner.run_with_output_and_cancel(
            &process_command,
            self.emit_output,
            &mut observe_cancel,
        )?;
        if cancelled {
            return Err(EditorExportBuildError::cancelled("CompileHost process"));
        }
        if execution.exit_code != Some(0) {
            return Err(EditorExportBuildError::WizardStageFailed {
                stage: ExportStage::CompileHost,
                exit_code: execution.exit_code,
            });
        }
        let stdout = execution.stdout_lines.join("\n").into_bytes();
        let stderr = execution.stderr_lines.join("\n").into_bytes();
        self.execution = Some(execution);
        Ok(ZirconBuildCommandExecution { stdout, stderr })
    }
}

#[cfg(test)]
#[path = "core_pipeline/tests/failure_receipt_tests.rs"]
mod failure_receipt_tests;

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
    ExportWizardCommandRunner, ProcessCommandRunner,
};

pub(super) fn run_core_compile_host(
    process_runner: &mut ProcessCommandRunner,
    command: &ExportWizardPipelineStageCommand,
    emit_output: &mut (dyn FnMut(ExportWizardCommandOutputLine) + Send),
    should_cancel: &mut (dyn FnMut() -> bool + Send),
) -> Result<ExportWizardCommandExecution, EditorExportBuildError> {
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
    let adapter = ExportWizardZirconBuildRunner {
        process_runner,
        emit_output,
        should_cancel,
        stage_command: command,
        execution: None,
    };
    let mut executor = ZirconBuildStageExecutor::new(preset.clone(), compile_host, adapter);
    let plan = ExportPipelinePlan::new([ExportStageNode::new(ExportStage::CompileHost, [])])
        .expect("single-stage CompileHost production graph is valid");
    let core_report_path = format!("{report_path}.core.json");
    let resume = load_core_pipeline_report(&core_report_path)?;
    let report = match plan.run(&mut executor, resume.as_ref()) {
        Ok(report) => report,
        Err(error) => {
            let (_report, source) = error.into_parts();
            return match source {
                crate::core::export::ZirconBuildStageExecutorError::Build(source) => Err(source),
                crate::core::export::ZirconBuildStageExecutorError::UnsupportedStage { stage } => {
                    Err(EditorExportBuildError::CoreUnsupportedStage { stage })
                }
                crate::core::export::ZirconBuildStageExecutorError::MissingCompileHostRecord => {
                    Err(EditorExportBuildError::CoreMissingCompileHostRecord)
                }
                crate::core::export::ZirconBuildStageExecutorError::EncodePreset(source) => {
                    Err(EditorExportBuildError::CorePresetFingerprint { source })
                }
                crate::core::export::ZirconBuildStageExecutorError::Fingerprint(source) => {
                    Err(EditorExportBuildError::CoreArtifactFingerprint { source })
                }
                crate::core::export::ZirconBuildStageExecutorError::BundleLayout(source) => {
                    Err(EditorExportBuildError::PlatformBundleLayout(source))
                }
            };
        }
    };
    write_core_pipeline_report(&core_report_path, &report)?;
    Ok(executor
        .into_runner()
        .execution
        .unwrap_or(ExportWizardCommandExecution {
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
            let (_report, source) = error.into_parts();
            match source {
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
            }
        })?;
    write_core_pipeline_report(&core_report_path, &report)
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

fn load_core_pipeline_report(
    path: &str,
) -> Result<Option<zircon_runtime_interface::export::ExportPipelineReport>, EditorExportBuildError>
{
    let path = std::path::Path::new(path);
    if !path.is_file() {
        return Ok(None);
    }
    let bytes = std::fs::read(path)
        .map_err(|source| EditorExportBuildError::CoreArtifactFingerprint { source })?;
    Ok(serde_json::from_slice(&bytes).ok())
}

fn write_core_pipeline_report(
    path: &str,
    report: &zircon_runtime_interface::export::ExportPipelineReport,
) -> Result<(), EditorExportBuildError> {
    if let Some(parent) = std::path::Path::new(path).parent() {
        std::fs::create_dir_all(parent)
            .map_err(|source| EditorExportBuildError::CoreArtifactFingerprint { source })?;
    }
    let bytes = serde_json::to_vec_pretty(report).map_err(|source| {
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

struct ExportWizardZirconBuildRunner<'a> {
    process_runner: &'a mut ProcessCommandRunner,
    emit_output: &'a mut (dyn FnMut(ExportWizardCommandOutputLine) + Send),
    should_cancel: &'a mut (dyn FnMut() -> bool + Send),
    stage_command: &'a ExportWizardPipelineStageCommand,
    execution: Option<ExportWizardCommandExecution>,
}

impl ZirconBuildCommandRunner for ExportWizardZirconBuildRunner<'_> {
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
        let execution = self.process_runner.run_with_output_and_cancel(
            &process_command,
            self.emit_output,
            self.should_cancel,
        )?;
        let stdout = execution.stdout_lines.join("\n").into_bytes();
        let stderr = execution.stderr_lines.join("\n").into_bytes();
        self.execution = Some(execution);
        Ok(ZirconBuildCommandExecution { stdout, stderr })
    }
}

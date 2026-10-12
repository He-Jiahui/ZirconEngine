use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::{Receiver, Sender};
use std::sync::Arc;

use crate::core::jobs::{test_job_system, EditorJobSystem};
use crate::ui::template_runtime::RetainedUiNodeProjection;
use zircon_runtime_interface::export::ExportStage;

use super::super::*;

pub(super) fn editor_jobs() -> EditorJobSystem {
    test_job_system()
}

#[derive(Default)]
pub(super) struct StubRunner {
    pub(super) executions: Vec<ExportWizardCommandExecution>,
    pub(super) seen_stages: Vec<ExportStage>,
}

impl StubRunner {
    pub(super) fn with_execution(execution: ExportWizardCommandExecution) -> Self {
        Self {
            executions: vec![execution],
            seen_stages: Vec::new(),
        }
    }
}

impl ExportWizardCommandRunner for StubRunner {
    fn run(
        &mut self,
        command: &ExportWizardPipelineStageCommand,
    ) -> Result<ExportWizardCommandExecution, EditorExportBuildError> {
        self.seen_stages.push(command.stage);
        if self.executions.is_empty() {
            return Ok(ExportWizardCommandExecution {
                exit_code: Some(0),
                stdout_lines: vec![command.stdout_banner("windows-release")],
                stderr_lines: Vec::new(),
            });
        }
        Ok(self.executions.remove(0))
    }
}

pub(super) struct CancelAfterRuns {
    requested_after_runs: usize,
    pub(super) observed_runs: Arc<AtomicUsize>,
}

impl CancelAfterRuns {
    pub(super) fn new(requested_after_runs: usize) -> Self {
        Self {
            requested_after_runs,
            observed_runs: Arc::new(AtomicUsize::new(0)),
        }
    }

    pub(super) fn observer(&self) -> Arc<AtomicUsize> {
        Arc::clone(&self.observed_runs)
    }
}

impl ExportWizardCancelSignal for CancelAfterRuns {
    fn is_cancel_requested(&self) -> bool {
        self.observed_runs.load(Ordering::Acquire) >= self.requested_after_runs
    }
}

pub(super) struct ObservingRunner {
    pub(super) inner: StubRunner,
    pub(super) observed_runs: Arc<AtomicUsize>,
}

impl ExportWizardCommandRunner for ObservingRunner {
    fn run(
        &mut self,
        command: &ExportWizardPipelineStageCommand,
    ) -> Result<ExportWizardCommandExecution, EditorExportBuildError> {
        let execution = self.inner.run(command);
        self.observed_runs.fetch_add(1, Ordering::Release);
        execution
    }
}

pub(super) struct BlockingRunner {
    stage_started: Sender<ExportStage>,
    release_stage: Receiver<()>,
    pub(super) seen_stages: Vec<ExportStage>,
}

impl BlockingRunner {
    pub(super) fn new(stage_started: Sender<ExportStage>, release_stage: Receiver<()>) -> Self {
        Self {
            stage_started,
            release_stage,
            seen_stages: Vec::new(),
        }
    }
}

impl ExportWizardCommandRunner for BlockingRunner {
    fn run(
        &mut self,
        command: &ExportWizardPipelineStageCommand,
    ) -> Result<ExportWizardCommandExecution, EditorExportBuildError> {
        self.seen_stages.push(command.stage);
        let _ = self.stage_started.send(command.stage);
        let _ = self.release_stage.recv();
        Ok(ExportWizardCommandExecution {
            exit_code: Some(0),
            stdout_lines: vec![command.stdout_banner("windows-release")],
            stderr_lines: Vec::new(),
        })
    }
}

pub(super) fn ready_export_options() -> ExportWizardPipelineOptions {
    let mut options = ExportWizardPipelineOptions::for_test_profile(
        "windows-release",
        "zircon-project.toml",
        "D:\\zircon-export",
    );
    options.source_asset_manifest = Some("D:\\assets\\source-assets.json".to_string());
    options.host_executable = Some("D:\\zircon-export\\host\\ZirconRuntime.exe".to_string());
    options
}

pub(super) fn stage_sequence(plan: &ExportWizardPipelinePlan) -> Vec<ExportStage> {
    plan.stages
        .iter()
        .map(|command| command.stage)
        .collect::<Vec<_>>()
}

pub(super) fn desktop_export_panel_template_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("editor crate should be inside repo root")
        .join("zircon_plugins")
        .join("editor_build_export_desktop")
        .join("editor")
        .join("panel.zui")
}

pub(super) fn find_projection_node<'a>(
    node: &'a RetainedUiNodeProjection,
    control_id: &str,
) -> Option<&'a RetainedUiNodeProjection> {
    if node.control_id.as_deref() == Some(control_id) {
        return Some(node);
    }
    node.children
        .iter()
        .find_map(|child| find_projection_node(child, control_id))
}

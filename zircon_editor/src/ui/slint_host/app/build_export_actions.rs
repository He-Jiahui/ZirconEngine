use std::collections::VecDeque;
use std::path::{Path, PathBuf};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    mpsc::{self, Receiver, Sender},
    Arc,
};
use std::thread;

use super::*;
use zircon_runtime::asset::project::ProjectManifest;
use zircon_runtime::plugin::{ExportPackagingStrategy, ExportProfile, ExportTargetPlatform};
use zircon_runtime::RuntimeTargetMode;

pub(super) const BUILD_EXPORT_ACTION_CONTROL_ID: &str = "BuildExportAction";

mod output_folder;

use output_folder::{pick_output_folder, reveal_path_in_file_browser, stable_picker_initial_dir};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum DesktopExportExecutionState {
    Exported,
    Failed,
    Cancelled,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct DesktopExportExecutionSummary {
    pub(super) profile_name: String,
    pub(super) output_root: PathBuf,
    pub(super) state: DesktopExportExecutionState,
    pub(super) invoked_cargo: bool,
    pub(super) generated_files: usize,
    pub(super) copied_packages: usize,
    pub(super) diagnostics: Vec<String>,
    pub(super) fatal_diagnostics: Vec<String>,
}

impl DesktopExportExecutionSummary {
    fn from_report(output_root: PathBuf, report: crate::ui::host::EditorExportBuildReport) -> Self {
        Self {
            profile_name: report.plan.profile.name,
            output_root,
            state: DesktopExportExecutionState::Exported,
            invoked_cargo: report.invoked_cargo,
            generated_files: report.generated_files.len(),
            copied_packages: report.copied_packages.len(),
            diagnostics: report.diagnostics,
            fatal_diagnostics: report.fatal_diagnostics,
        }
    }

    fn failed(profile_name: impl Into<String>, output_root: PathBuf, error: String) -> Self {
        Self {
            profile_name: profile_name.into(),
            output_root,
            state: DesktopExportExecutionState::Failed,
            invoked_cargo: false,
            generated_files: 0,
            copied_packages: 0,
            diagnostics: Vec::new(),
            fatal_diagnostics: vec![error],
        }
    }

    fn cancelled(profile_name: impl Into<String>, output_root: PathBuf, reason: String) -> Self {
        Self {
            profile_name: profile_name.into(),
            output_root,
            state: DesktopExportExecutionState::Cancelled,
            invoked_cargo: false,
            generated_files: 0,
            copied_packages: 0,
            diagnostics: vec![reason],
            fatal_diagnostics: Vec::new(),
        }
    }

    pub(super) fn fatal(&self) -> bool {
        !self.fatal_diagnostics.is_empty()
    }

    fn status_label(&self) -> &'static str {
        match self.state {
            DesktopExportExecutionState::Exported => "Exported",
            DesktopExportExecutionState::Failed => "Failed",
            DesktopExportExecutionState::Cancelled => "Cancelled",
        }
    }

    fn status_message(&self) -> String {
        if self.state == DesktopExportExecutionState::Cancelled {
            return format!(
                "Export {} cancelled -> {}",
                self.profile_name,
                self.output_root.display()
            );
        }
        if self.fatal() {
            return format!(
                "Export {} failed: {}",
                self.profile_name,
                self.fatal_diagnostics.join("; ")
            );
        }
        let cargo = if self.invoked_cargo {
            "cargo build invoked"
        } else {
            "cargo build skipped"
        };
        format!(
            "Export {} finished: {} files, {} native packages, {cargo} -> {}",
            self.profile_name,
            self.generated_files,
            self.copied_packages,
            self.output_root.display()
        )
    }

    pub(super) fn pane_diagnostics(&self) -> String {
        let mut lines = Vec::new();
        lines.push(format!(
            "Last export output: {}",
            self.output_root.display()
        ));
        lines.push(if self.invoked_cargo {
            "Last export invoked Cargo".to_string()
        } else {
            "Last export skipped Cargo".to_string()
        });
        lines.extend(self.fatal_diagnostics.iter().cloned());
        lines.extend(self.diagnostics.iter().take(6).cloned());
        lines.join("\n")
    }
}

pub(super) enum BuildExportAction<'a> {
    Execute {
        profile_name: &'a str,
    },
    Cancel {
        profile_name: &'a str,
    },
    SetOutput {
        profile_name: &'a str,
        output_root: &'a str,
    },
    ChooseOutput {
        profile_name: &'a str,
    },
    ClearOutput {
        profile_name: &'a str,
    },
    RevealOutput {
        profile_name: &'a str,
    },
}

pub(super) fn parse_build_export_action(action_id: &str) -> Option<BuildExportAction<'_>> {
    if let Some(profile_name) = action_id
        .strip_prefix("BuildExport.Execute.")
        .filter(|profile_name| !profile_name.trim().is_empty())
    {
        return Some(BuildExportAction::Execute { profile_name });
    }
    if let Some(profile_name) = action_id
        .strip_prefix("BuildExport.Cancel.")
        .filter(|profile_name| !profile_name.trim().is_empty())
    {
        return Some(BuildExportAction::Cancel { profile_name });
    }
    if let Some(profile_name) = action_id
        .strip_prefix("BuildExport.ClearOutput.")
        .filter(|profile_name| !profile_name.trim().is_empty())
    {
        return Some(BuildExportAction::ClearOutput { profile_name });
    }
    if let Some(profile_name) = action_id
        .strip_prefix("BuildExport.RevealOutput.")
        .filter(|profile_name| !profile_name.trim().is_empty())
    {
        return Some(BuildExportAction::RevealOutput { profile_name });
    }
    if let Some(profile_name) = action_id
        .strip_prefix("BuildExport.ChooseOutput.")
        .filter(|profile_name| !profile_name.trim().is_empty())
    {
        return Some(BuildExportAction::ChooseOutput { profile_name });
    }
    action_id
        .strip_prefix("BuildExport.SetOutput.")
        .and_then(|rest| rest.split_once('|'))
        .and_then(|(profile_name, output_root)| {
            if profile_name.trim().is_empty() || output_root.trim().is_empty() {
                None
            } else {
                Some(BuildExportAction::SetOutput {
                    profile_name,
                    output_root,
                })
            }
        })
}

pub(super) fn desktop_export_profiles() -> Vec<ExportProfile> {
    let desktop = [
        ("desktop_windows", ExportTargetPlatform::Windows),
        ("desktop_linux", ExportTargetPlatform::Linux),
        ("desktop_macos", ExportTargetPlatform::Macos),
    ]
    .into_iter()
    .map(|(name, platform)| {
        ExportProfile::new(name, RuntimeTargetMode::ClientRuntime, platform).with_strategies([
            ExportPackagingStrategy::SourceTemplate,
            ExportPackagingStrategy::LibraryEmbed,
            ExportPackagingStrategy::NativeDynamic,
        ])
    });
    let platform_scaffolds = [
        ("mobile_android", ExportTargetPlatform::Android),
        ("mobile_ios", ExportTargetPlatform::Ios),
        ("browser_webgpu", ExportTargetPlatform::WebGpu),
        ("browser_wasm", ExportTargetPlatform::Wasm),
    ]
    .into_iter()
    .map(|(name, platform)| {
        ExportProfile::new(name, RuntimeTargetMode::ClientRuntime, platform).with_strategies([
            ExportPackagingStrategy::SourceTemplate,
            ExportPackagingStrategy::LibraryEmbed,
        ])
    });

    desktop.chain(platform_scaffolds).collect()
}

pub(super) fn desktop_export_profile(profile_name: &str) -> Option<ExportProfile> {
    desktop_export_profiles()
        .into_iter()
        .find(|profile| profile.name == profile_name)
}

pub(super) fn export_platform_label(platform: ExportTargetPlatform) -> &'static str {
    match platform {
        ExportTargetPlatform::Windows => "Windows",
        ExportTargetPlatform::Linux => "Linux",
        ExportTargetPlatform::Macos => "macOS",
        ExportTargetPlatform::Android => "Android",
        ExportTargetPlatform::Ios => "iOS",
        ExportTargetPlatform::WebGpu => "WebGPU",
        ExportTargetPlatform::Wasm => "WASM",
    }
}

pub(super) fn default_desktop_export_output_root(
    project_root: &Path,
    profile_name: &str,
) -> PathBuf {
    project_root
        .join("Builds")
        .join("zircon")
        .join(profile_name)
}

pub(super) fn apply_summary_to_target(
    target: &mut crate::ui::layouts::windows::workbench_host_window::BuildExportTargetViewData,
    summary: &DesktopExportExecutionSummary,
) {
    target.status = summary.status_label().into();
    target.generated_files = summary.generated_files.to_string().into();
    target.native_dynamic_packages = summary.copied_packages.to_string().into();
    target.diagnostics = summary.pane_diagnostics().into();
    target.fatal = target.fatal || summary.fatal();
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum DesktopExportJobPhase {
    Queued,
    Running,
    CancelRequested,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct DesktopExportJobSnapshot {
    pub(super) id: u64,
    pub(super) profile_name: String,
    pub(super) output_root: PathBuf,
    pub(super) phase: DesktopExportJobPhase,
    pub(super) progress: Option<DesktopExportProgressSnapshot>,
}

impl DesktopExportJobSnapshot {
    fn status_label(&self) -> &'static str {
        match self.phase {
            DesktopExportJobPhase::Queued => "Queued",
            DesktopExportJobPhase::Running => "Running",
            DesktopExportJobPhase::CancelRequested => "Cancel requested",
        }
    }

    fn pane_diagnostics(&self) -> String {
        let phase = match self.phase {
            DesktopExportJobPhase::Queued => "waiting for the current desktop export job",
            DesktopExportJobPhase::Running => "export backend is running",
            DesktopExportJobPhase::CancelRequested => {
                "cancel requested; backend result will be ignored when it returns"
            }
        };
        let mut lines = vec![
            format!("Output: {}", self.output_root.display()),
            format!("Progress: {phase}"),
        ];
        if let Some(progress) = &self.progress {
            lines.push(progress.pane_diagnostic());
        }
        lines.join("\n")
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct DesktopExportProgressSnapshot {
    pub(super) stage: String,
    pub(super) percent: u8,
    pub(super) message: String,
}

impl DesktopExportProgressSnapshot {
    fn from_report(progress: crate::ui::host::EditorExportBuildProgress) -> Self {
        Self {
            stage: progress.stage,
            percent: progress.percent,
            message: progress.message,
        }
    }

    fn pane_diagnostic(&self) -> String {
        format!("Stage: {}% {} - {}", self.percent, self.stage, self.message)
    }
}

pub(super) fn apply_job_snapshot_to_target(
    target: &mut crate::ui::layouts::windows::workbench_host_window::BuildExportTargetViewData,
    snapshot: &DesktopExportJobSnapshot,
) {
    target.status = snapshot.status_label().into();
    target.diagnostics = snapshot.pane_diagnostics().into();
    target.fatal = false;
}

#[derive(Debug, PartialEq, Eq)]
pub(super) enum DesktopExportCancellation {
    NotFound,
    PendingCancelled(DesktopExportExecutionSummary),
    ActiveCancelRequested(DesktopExportJobSnapshot),
}

#[derive(Debug)]
struct DesktopExportQueuedJob {
    id: u64,
    profile_name: String,
    project_root: PathBuf,
    manifest: ProjectManifest,
    output_root: PathBuf,
    cancel_requested: Arc<AtomicBool>,
}

#[derive(Debug)]
struct DesktopExportActiveJob {
    id: u64,
    profile_name: String,
    output_root: PathBuf,
    cancel_requested: Arc<AtomicBool>,
    progress: Option<DesktopExportProgressSnapshot>,
}

#[derive(Debug)]
struct DesktopExportJobResult {
    id: u64,
    profile_name: String,
    output_root: PathBuf,
    cancel_requested: Arc<AtomicBool>,
    result: Result<crate::ui::host::EditorExportBuildReport, String>,
}

#[derive(Debug)]
struct DesktopExportJobProgress {
    id: u64,
    progress: DesktopExportProgressSnapshot,
}

#[derive(Debug)]
enum DesktopExportJobMessage {
    Progress(DesktopExportJobProgress),
    Finished(DesktopExportJobResult),
}

pub(super) struct DesktopExportJobQueue {
    next_id: u64,
    pending: VecDeque<DesktopExportQueuedJob>,
    active: Option<DesktopExportActiveJob>,
    sender: Sender<DesktopExportJobMessage>,
    receiver: Receiver<DesktopExportJobMessage>,
}

impl Default for DesktopExportJobQueue {
    fn default() -> Self {
        let (sender, receiver) = mpsc::channel();
        Self {
            next_id: 1,
            pending: VecDeque::new(),
            active: None,
            sender,
            receiver,
        }
    }
}

impl DesktopExportJobQueue {
    pub(super) fn enqueue(
        &mut self,
        profile_name: impl Into<String>,
        project_root: PathBuf,
        manifest: ProjectManifest,
        output_root: PathBuf,
    ) -> DesktopExportJobSnapshot {
        let id = self.next_id;
        self.next_id += 1;
        let profile_name = profile_name.into();
        let snapshot = DesktopExportJobSnapshot {
            id,
            profile_name: profile_name.clone(),
            output_root: output_root.clone(),
            phase: DesktopExportJobPhase::Queued,
            progress: None,
        };
        self.pending.push_back(DesktopExportQueuedJob {
            id,
            profile_name,
            project_root,
            manifest,
            output_root,
            cancel_requested: Arc::new(AtomicBool::new(false)),
        });
        snapshot
    }

    pub(super) fn is_profile_busy(&self, profile_name: &str) -> bool {
        self.active
            .as_ref()
            .is_some_and(|active| active.profile_name == profile_name)
            || self
                .pending
                .iter()
                .any(|pending| pending.profile_name == profile_name)
    }

    pub(super) fn snapshots(&self) -> Vec<DesktopExportJobSnapshot> {
        let mut snapshots = Vec::new();
        if let Some(active) = &self.active {
            snapshots.push(DesktopExportJobSnapshot {
                id: active.id,
                profile_name: active.profile_name.clone(),
                output_root: active.output_root.clone(),
                phase: if active.cancel_requested.load(Ordering::SeqCst) {
                    DesktopExportJobPhase::CancelRequested
                } else {
                    DesktopExportJobPhase::Running
                },
                progress: active.progress.clone(),
            });
        }
        snapshots.extend(self.pending.iter().map(|pending| DesktopExportJobSnapshot {
            id: pending.id,
            profile_name: pending.profile_name.clone(),
            output_root: pending.output_root.clone(),
            phase: DesktopExportJobPhase::Queued,
            progress: None,
        }));
        snapshots
    }

    pub(super) fn cancel_profile(&mut self, profile_name: &str) -> DesktopExportCancellation {
        if let Some(index) = self
            .pending
            .iter()
            .position(|pending| pending.profile_name == profile_name)
        {
            let pending = self
                .pending
                .remove(index)
                .expect("pending job index came from this queue");
            pending.cancel_requested.store(true, Ordering::SeqCst);
            return DesktopExportCancellation::PendingCancelled(
                DesktopExportExecutionSummary::cancelled(
                    pending.profile_name,
                    pending.output_root,
                    "Queued export was cancelled before it started".to_string(),
                ),
            );
        }

        if let Some(active) = self
            .active
            .as_ref()
            .filter(|active| active.profile_name == profile_name)
        {
            active.cancel_requested.store(true, Ordering::SeqCst);
            return DesktopExportCancellation::ActiveCancelRequested(DesktopExportJobSnapshot {
                id: active.id,
                profile_name: active.profile_name.clone(),
                output_root: active.output_root.clone(),
                phase: DesktopExportJobPhase::CancelRequested,
                progress: active.progress.clone(),
            });
        }

        DesktopExportCancellation::NotFound
    }

    pub(super) fn poll_updates(&mut self) -> (Vec<DesktopExportExecutionSummary>, bool) {
        let mut summaries = Vec::new();
        let mut changed = false;
        while let Ok(message) = self.receiver.try_recv() {
            match message {
                DesktopExportJobMessage::Progress(progress) => {
                    if let Some(active) = self
                        .active
                        .as_mut()
                        .filter(|active| active.id == progress.id)
                    {
                        active.progress = Some(progress.progress);
                        changed = true;
                    }
                }
                DesktopExportJobMessage::Finished(result) => {
                    if self
                        .active
                        .as_ref()
                        .is_some_and(|active| active.id == result.id)
                    {
                        self.active = None;
                    }
                    summaries.push(desktop_export_summary_from_job_result(result));
                    changed = true;
                }
            }
        }
        (summaries, changed)
    }

    pub(super) fn start_next(
        &mut self,
        editor_manager: Arc<crate::ui::host::EditorManager>,
    ) -> Option<DesktopExportJobSnapshot> {
        if self.active.is_some() {
            return None;
        }
        let job = self.pending.pop_front()?;
        if job.cancel_requested.load(Ordering::SeqCst) {
            return None;
        }
        let snapshot = DesktopExportJobSnapshot {
            id: job.id,
            profile_name: job.profile_name.clone(),
            output_root: job.output_root.clone(),
            phase: DesktopExportJobPhase::Running,
            progress: Some(DesktopExportProgressSnapshot {
                stage: "queued".to_string(),
                percent: 0,
                message: "Waiting for export runner to start".to_string(),
            }),
        };
        self.active = Some(DesktopExportActiveJob {
            id: job.id,
            profile_name: job.profile_name.clone(),
            output_root: job.output_root.clone(),
            cancel_requested: job.cancel_requested.clone(),
            progress: snapshot.progress.clone(),
        });

        let sender = self.sender.clone();
        thread::spawn(move || {
            let progress_sender = sender.clone();
            let job_id = job.id;
            let result = editor_manager
                .execute_native_aware_export_build_with_cancellation_and_progress(
                    &job.project_root,
                    &job.output_root,
                    &job.manifest,
                    &job.profile_name,
                    Some(job.cancel_requested.as_ref()),
                    move |progress| {
                        let _ = progress_sender.send(DesktopExportJobMessage::Progress(
                            DesktopExportJobProgress {
                                id: job_id,
                                progress: DesktopExportProgressSnapshot::from_report(progress),
                            },
                        ));
                    },
                );
            let _ = sender.send(DesktopExportJobMessage::Finished(DesktopExportJobResult {
                id: job.id,
                profile_name: job.profile_name,
                output_root: job.output_root,
                cancel_requested: job.cancel_requested,
                result,
            }));
        });
        Some(snapshot)
    }
}

fn desktop_export_summary_from_job_result(
    result: DesktopExportJobResult,
) -> DesktopExportExecutionSummary {
    if result.cancel_requested.load(Ordering::SeqCst) {
        return DesktopExportExecutionSummary::cancelled(
            result.profile_name,
            result.output_root,
            "Export result ignored because cancellation was requested while it was running"
                .to_string(),
        );
    }
    match result.result {
        Ok(report) => DesktopExportExecutionSummary::from_report(result.output_root, report),
        Err(error) => {
            DesktopExportExecutionSummary::failed(result.profile_name, result.output_root, error)
        }
    }
}

impl SlintEditorHost {
    pub(super) fn poll_desktop_export_jobs(&mut self) {
        let (summaries, mut changed) = self.desktop_export_jobs.poll_updates();
        for summary in summaries {
            let message = summary.status_message();
            self.desktop_export_reports
                .insert(summary.profile_name.clone(), summary);
            self.set_status_line(message);
        }
        if let Some(started) = self
            .desktop_export_jobs
            .start_next(self.editor_manager.clone())
        {
            self.set_status_line(format!(
                "Desktop export {} started -> {}",
                started.profile_name,
                started.output_root.display()
            ));
            changed = true;
        }
        if changed {
            self.layout_dirty = true;
            self.presentation_dirty = true;
        }
    }

    pub(super) fn dispatch_build_export_action(&mut self, action_id: &str) {
        let Some(action) = parse_build_export_action(action_id) else {
            self.set_status_line(format!("Unknown build/export action {action_id}"));
            return;
        };

        match action {
            BuildExportAction::Execute { profile_name } => {
                self.enqueue_desktop_export(profile_name);
            }
            BuildExportAction::Cancel { profile_name } => {
                self.cancel_desktop_export(profile_name);
            }
            BuildExportAction::SetOutput {
                profile_name,
                output_root,
            } => {
                self.desktop_export_output_overrides
                    .insert(profile_name.to_string(), PathBuf::from(output_root));
                self.layout_dirty = true;
                self.set_status_line(format!(
                    "Desktop export output for {profile_name} set to {output_root}"
                ));
            }
            BuildExportAction::ClearOutput { profile_name } => {
                self.desktop_export_output_overrides.remove(profile_name);
                self.layout_dirty = true;
                self.set_status_line(format!(
                    "Desktop export output for {profile_name} reset to project default"
                ));
            }
            BuildExportAction::ChooseOutput { profile_name } => {
                self.choose_desktop_export_output(profile_name);
            }
            BuildExportAction::RevealOutput { profile_name } => {
                self.reveal_desktop_export_output(profile_name);
            }
        }
    }

    fn enqueue_desktop_export(&mut self, profile_name: &str) {
        if self.desktop_export_jobs.is_profile_busy(profile_name) {
            self.set_status_line(format!("Desktop export {profile_name} is already queued"));
            return;
        }
        let project_path = self.runtime.editor_snapshot().project_path;
        let result = crate::ui::workbench::project::project_root_path(&project_path)
            .map_err(|error| error.to_string())
            .and_then(|project_root| {
                let profile = desktop_export_profile(profile_name)
                    .ok_or_else(|| format!("unknown desktop export profile {profile_name}"))?;
                let manifest_path = project_root.join("zircon-project.toml");
                let mut manifest =
                    ProjectManifest::load(&manifest_path).map_err(|error| error.to_string())?;
                manifest.export_profiles.push(profile);
                let output_root =
                    self.effective_desktop_export_output_root(&project_root, profile_name);
                Ok((project_root, manifest, output_root))
            });

        match result {
            Ok((project_root, manifest, output_root)) => {
                let snapshot = self.desktop_export_jobs.enqueue(
                    profile_name,
                    project_root,
                    manifest,
                    output_root,
                );
                self.layout_dirty = true;
                self.set_status_line(format!(
                    "Desktop export {} queued -> {}",
                    snapshot.profile_name,
                    snapshot.output_root.display()
                ));
                self.poll_desktop_export_jobs();
            }
            Err(error) => self.set_status_line(format!("Build/export action failed: {error}")),
        }
    }

    fn cancel_desktop_export(&mut self, profile_name: &str) {
        match self.desktop_export_jobs.cancel_profile(profile_name) {
            DesktopExportCancellation::NotFound => self.set_status_line(format!(
                "No queued or running desktop export for {profile_name}"
            )),
            DesktopExportCancellation::PendingCancelled(summary) => {
                let message = summary.status_message();
                self.desktop_export_reports
                    .insert(summary.profile_name.clone(), summary);
                self.layout_dirty = true;
                self.set_status_line(message);
            }
            DesktopExportCancellation::ActiveCancelRequested(snapshot) => {
                self.layout_dirty = true;
                self.set_status_line(format!(
                    "Cancel requested for desktop export {}",
                    snapshot.profile_name
                ));
            }
        }
    }

    fn choose_desktop_export_output(&mut self, profile_name: &str) {
        let project_path = self.runtime.editor_snapshot().project_path;
        let result = crate::ui::workbench::project::project_root_path(&project_path)
            .map_err(|error| error.to_string())
            .and_then(|project_root| {
                let current_output =
                    self.effective_desktop_export_output_root(&project_root, profile_name);
                let initial_dir = stable_picker_initial_dir(&current_output, &project_root);
                pick_output_folder(&initial_dir)
            });

        match result {
            Ok(Some(output_root)) => {
                self.desktop_export_output_overrides
                    .insert(profile_name.to_string(), output_root.clone());
                self.layout_dirty = true;
                self.set_status_line(format!(
                    "Desktop export output for {profile_name} set to {}",
                    output_root.display()
                ));
            }
            Ok(None) => self.set_status_line(format!(
                "Desktop export output picker cancelled for {profile_name}"
            )),
            Err(error) => self.set_status_line(format!("Build/export action failed: {error}")),
        }
    }

    fn reveal_desktop_export_output(&mut self, profile_name: &str) {
        let project_path = self.runtime.editor_snapshot().project_path;
        let result = crate::ui::workbench::project::project_root_path(&project_path)
            .map_err(|error| error.to_string())
            .and_then(|project_root| {
                let output_root =
                    self.effective_desktop_export_output_root(&project_root, profile_name);
                std::fs::create_dir_all(&output_root).map_err(|error| {
                    format!(
                        "failed to create desktop export output folder {}: {error}",
                        output_root.display()
                    )
                })?;
                reveal_path_in_file_browser(&output_root)?;
                Ok(output_root)
            });

        match result {
            Ok(output_root) => self.set_status_line(format!(
                "Desktop export output for {profile_name} opened -> {}",
                output_root.display()
            )),
            Err(error) => self.set_status_line(format!("Build/export action failed: {error}")),
        }
    }

    pub(super) fn effective_desktop_export_output_root(
        &self,
        project_root: &Path,
        profile_name: &str,
    ) -> PathBuf {
        self.desktop_export_output_overrides
            .get(profile_name)
            .cloned()
            .unwrap_or_else(|| default_desktop_export_output_root(project_root, profile_name))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_export_actions_parse_execute_profile() {
        match parse_build_export_action("BuildExport.Execute.desktop_windows") {
            Some(BuildExportAction::Execute { profile_name }) => {
                assert_eq!(profile_name, "desktop_windows");
            }
            _ => panic!("execute action should parse"),
        }
        match parse_build_export_action("BuildExport.Cancel.desktop_windows") {
            Some(BuildExportAction::Cancel { profile_name }) => {
                assert_eq!(profile_name, "desktop_windows");
            }
            _ => panic!("cancel action should parse"),
        }
        match parse_build_export_action("BuildExport.SetOutput.desktop_windows|D:/Builds/Zircon") {
            Some(BuildExportAction::SetOutput {
                profile_name,
                output_root,
            }) => {
                assert_eq!(profile_name, "desktop_windows");
                assert_eq!(output_root, "D:/Builds/Zircon");
            }
            _ => panic!("set-output action should parse"),
        }
        match parse_build_export_action("BuildExport.ChooseOutput.desktop_windows") {
            Some(BuildExportAction::ChooseOutput { profile_name }) => {
                assert_eq!(profile_name, "desktop_windows");
            }
            _ => panic!("choose-output action should parse"),
        }
        match parse_build_export_action("BuildExport.ClearOutput.desktop_windows") {
            Some(BuildExportAction::ClearOutput { profile_name }) => {
                assert_eq!(profile_name, "desktop_windows");
            }
            _ => panic!("clear-output action should parse"),
        }
        match parse_build_export_action("BuildExport.RevealOutput.desktop_windows") {
            Some(BuildExportAction::RevealOutput { profile_name }) => {
                assert_eq!(profile_name, "desktop_windows");
            }
            _ => panic!("reveal-output action should parse"),
        }
        assert!(parse_build_export_action("BuildExport.Execute.").is_none());
        assert!(parse_build_export_action("BuildExport.Unknown.desktop_windows").is_none());
    }

    #[test]
    fn desktop_export_output_root_is_project_local_and_profile_scoped() {
        let root = Path::new("Project");
        assert_eq!(
            default_desktop_export_output_root(root, "desktop_linux"),
            PathBuf::from("Project")
                .join("Builds")
                .join("zircon")
                .join("desktop_linux")
        );
    }

    #[test]
    fn build_export_profiles_include_mobile_and_browser_source_scaffolds() {
        let profiles = desktop_export_profiles();
        let android = profiles
            .iter()
            .find(|profile| profile.name == "mobile_android")
            .expect("mobile Android export profile is projected");
        let webgpu = profiles
            .iter()
            .find(|profile| profile.name == "browser_webgpu")
            .expect("WebGPU export profile is projected");

        assert_eq!(android.target_platform, ExportTargetPlatform::Android);
        assert_eq!(webgpu.target_platform, ExportTargetPlatform::WebGpu);
        assert!(android.uses_strategy(ExportPackagingStrategy::SourceTemplate));
        assert!(android.uses_strategy(ExportPackagingStrategy::LibraryEmbed));
        assert!(!android.uses_strategy(ExportPackagingStrategy::NativeDynamic));
        assert!(!webgpu.uses_strategy(ExportPackagingStrategy::NativeDynamic));
    }

    #[test]
    fn desktop_export_job_queue_starts_and_cancels_pending_jobs() {
        let mut queue = DesktopExportJobQueue::default();
        let first = queue.enqueue(
            "desktop_windows",
            PathBuf::from("Project"),
            ProjectManifest::new(
                "Project",
                zircon_runtime::asset::AssetUri::parse("res://main.scene.toml")
                    .expect("test asset URI is valid"),
                1,
            ),
            PathBuf::from("Builds/windows"),
        );
        let second = queue.enqueue(
            "desktop_linux",
            PathBuf::from("Project"),
            ProjectManifest::new(
                "Project",
                zircon_runtime::asset::AssetUri::parse("res://main.scene.toml")
                    .expect("test asset URI is valid"),
                1,
            ),
            PathBuf::from("Builds/linux"),
        );
        assert_eq!(first.phase, DesktopExportJobPhase::Queued);
        assert_eq!(second.id, first.id + 1);
        assert!(queue.is_profile_busy("desktop_windows"));

        match queue.cancel_profile("desktop_linux") {
            DesktopExportCancellation::PendingCancelled(summary) => {
                assert_eq!(summary.profile_name, "desktop_linux");
                assert_eq!(summary.state, DesktopExportExecutionState::Cancelled);
            }
            other => panic!("expected pending cancellation, got {other:?}"),
        }
        assert!(!queue.is_profile_busy("desktop_linux"));
    }

    #[test]
    fn desktop_export_job_snapshot_projects_stage_progress() {
        let mut target =
            crate::ui::layouts::windows::workbench_host_window::BuildExportTargetViewData {
                profile_name: "desktop_windows".into(),
                status: "Ready".into(),
                diagnostics: "".into(),
                fatal: true,
                ..Default::default()
            };
        let snapshot = DesktopExportJobSnapshot {
            id: 7,
            profile_name: "desktop_windows".to_string(),
            output_root: PathBuf::from("Builds/windows"),
            phase: DesktopExportJobPhase::Running,
            progress: Some(DesktopExportProgressSnapshot {
                stage: "cargo-build".to_string(),
                percent: 72,
                message: "Running generated SourceTemplate Cargo build".to_string(),
            }),
        };

        apply_job_snapshot_to_target(&mut target, &snapshot);

        assert_eq!(target.status.as_str(), "Running");
        assert!(!target.fatal);
        assert!(target
            .diagnostics
            .as_str()
            .contains("Stage: 72% cargo-build"));
        assert!(target
            .diagnostics
            .as_str()
            .contains("Running generated SourceTemplate Cargo build"));
    }
}

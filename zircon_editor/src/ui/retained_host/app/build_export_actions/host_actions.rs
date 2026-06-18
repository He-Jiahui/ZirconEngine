use std::path::{Path, PathBuf};

use crate::ui::workbench::project::project_root_path;
use zircon_runtime::asset::project::ProjectManifest;

use super::output_folder::{
    pick_output_folder, reveal_path_in_file_browser, stable_picker_initial_dir,
};
use super::{
    default_desktop_export_output_root, desktop_export_profile,
    desktop_export_status_task_from_queue, parse_build_export_action, BuildExportAction,
    DesktopExportCancellation,
};
use crate::ui::retained_host::app::RetainedEditorHost;

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app) fn poll_desktop_export_jobs(&mut self) {
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
        self.sync_desktop_export_status_task();
        if changed {
            self.mark_layout_dirty();
        }
    }

    pub(in crate::ui::retained_host::app) fn dispatch_build_export_action(
        &mut self,
        action_id: &str,
    ) {
        let Some(action) = parse_build_export_action(action_id) else {
            self.set_status_line(format!("Unknown build/export action {action_id}"));
            return;
        };

        match action {
            BuildExportAction::GeneratePlan { profile_name } => {
                self.mark_layout_dirty();
                self.set_status_line(format!("Desktop export plan for {profile_name} refreshed"));
            }
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
                self.mark_layout_dirty();
                self.set_status_line(format!(
                    "Desktop export output for {profile_name} set to {output_root}"
                ));
            }
            BuildExportAction::ClearOutput { profile_name } => {
                self.desktop_export_output_overrides.remove(profile_name);
                self.mark_layout_dirty();
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
        let result = project_root_path(&project_path)
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
                self.mark_layout_dirty();
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
                self.mark_layout_dirty();
                self.set_status_line(message);
                self.sync_desktop_export_status_task();
            }
            DesktopExportCancellation::ActiveCancelRequested(snapshot) => {
                self.mark_layout_dirty();
                self.set_status_line(format!(
                    "Cancel requested for desktop export {}",
                    snapshot.profile_name
                ));
                self.sync_desktop_export_status_task();
            }
        }
    }

    fn sync_desktop_export_status_task(&mut self) {
        self.set_status_task_progress(desktop_export_status_task_from_queue(
            &self.desktop_export_jobs,
        ));
    }

    fn choose_desktop_export_output(&mut self, profile_name: &str) {
        let project_path = self.runtime.editor_snapshot().project_path;
        let result = project_root_path(&project_path)
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
                self.mark_layout_dirty();
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
        let result = project_root_path(&project_path)
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

    pub(in crate::ui::retained_host::app) fn effective_desktop_export_output_root(
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

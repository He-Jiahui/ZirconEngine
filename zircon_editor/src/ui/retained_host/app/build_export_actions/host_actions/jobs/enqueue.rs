use crate::ui::retained_host::app::RetainedEditorHost;
use crate::ui::workbench::project::project_root_path;
use zircon_runtime::asset::project::ProjectManifest;

use super::super::super::DesktopExportActionError;
use super::super::super::desktop_export_profile;

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app::build_export_actions::host_actions) fn enqueue_desktop_export(
        &mut self,
        profile_name: &str,
    ) {
        if self.desktop_export_jobs.is_profile_busy(profile_name) {
            self.set_status_line(format!("Desktop export {profile_name} is already queued"));
            return;
        }
        let project_path = self.runtime.editor_snapshot().project_path;
        let result = project_root_path(&project_path)
            .map_err(DesktopExportActionError::from)
            .and_then(|project_root| {
                let profile = desktop_export_profile(profile_name).ok_or_else(|| {
                    DesktopExportActionError::UnknownProfile {
                        profile_name: profile_name.to_string(),
                    }
                })?;
                let manifest_path = project_root.join("zircon-project.toml");
                let mut manifest = ProjectManifest::load(&manifest_path)
                    .map_err(|source| DesktopExportActionError::Manifest { source })?;
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
                self.desktop_export_wizard_sessions
                    .invalidate_projection_overlay();
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
}

use std::path::{Path, PathBuf};

use crate::ui::retained_host::app::RetainedEditorHost;
use crate::ui::workbench::project::project_root_path;

use super::super::default_desktop_export_output_root;
use super::super::output_folder::{
    pick_output_folder, reveal_path_in_file_browser, stable_picker_initial_dir,
};
use super::super::DesktopExportActionError;

impl RetainedEditorHost {
    pub(super) fn choose_desktop_export_output(&mut self, profile_name: &str) {
        let project_path = self.runtime.editor_snapshot().project_path;
        let result = project_root_path(&project_path)
            .map_err(DesktopExportActionError::from)
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

    pub(super) fn reveal_desktop_export_output(&mut self, profile_name: &str) {
        let project_path = self.runtime.editor_snapshot().project_path;
        let result = project_root_path(&project_path)
            .map_err(DesktopExportActionError::from)
            .and_then(|project_root| {
                let output_root =
                    self.effective_desktop_export_output_root(&project_root, profile_name);
                std::fs::create_dir_all(&output_root).map_err(|source| {
                    DesktopExportActionError::CreateOutput {
                        path: output_root.clone(),
                        source,
                    }
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

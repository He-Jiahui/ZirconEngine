use std::path::PathBuf;

use super::{parse_build_export_action, BuildExportAction};
use crate::ui::retained_host::app::RetainedEditorHost;

mod jobs;
mod output;

impl RetainedEditorHost {
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
                self.desktop_export_wizard_sessions
                    .invalidate_projection_source();
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
                self.desktop_export_wizard_sessions
                    .invalidate_projection_overlay();
                self.mark_layout_dirty();
                self.set_status_line(format!(
                    "Desktop export output for {profile_name} set to {output_root}"
                ));
            }
            BuildExportAction::ClearOutput { profile_name } => {
                self.desktop_export_output_overrides.remove(profile_name);
                self.desktop_export_wizard_sessions
                    .invalidate_projection_overlay();
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
}

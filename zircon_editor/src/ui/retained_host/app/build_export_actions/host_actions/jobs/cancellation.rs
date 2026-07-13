use crate::ui::retained_host::app::RetainedEditorHost;

use super::super::super::DesktopExportCancellation;

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app::build_export_actions::host_actions) fn cancel_desktop_export(
        &mut self,
        profile_name: &str,
    ) {
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
            }
            DesktopExportCancellation::ActiveCancelRequested(snapshot) => {
                self.mark_layout_dirty();
                self.set_status_line(format!(
                    "Cancel requested for desktop export {}",
                    snapshot.profile_name
                ));
            }
        }
    }
}

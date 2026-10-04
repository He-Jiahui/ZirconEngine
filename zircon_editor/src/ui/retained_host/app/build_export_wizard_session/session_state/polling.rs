use crate::ui::host::{
    ExportWizardPanelRequest, ExportWizardPanelSessionError, ExportWizardPanelUpdate,
};

use super::DesktopExportWizardSessions;

impl DesktopExportWizardSessions {
    pub(in crate::ui::retained_host::app) fn poll_all(
        &mut self,
    ) -> Vec<(
        String,
        Result<ExportWizardPanelUpdate, ExportWizardPanelSessionError>,
    )> {
        let mut updates = Vec::new();
        for (profile_name, session) in &mut self.sessions {
            if session.view_model().snapshot().is_terminal() {
                continue;
            }
            let before = session.view_model().snapshot().clone();
            let result = session.handle_request(ExportWizardPanelRequest::Poll);
            let changed = match &result {
                Ok(update) => update.events_drained > 0 || before != update.snapshot,
                Err(_) => true,
            };
            if changed {
                updates.push((profile_name.clone(), result));
            }
        }
        if !updates.is_empty() {
            self.invalidate_projection_overlay();
        }
        updates.sort_unstable_by(|(left, _), (right, _)| left.cmp(right));
        updates
    }
}

#[cfg(test)]
#[path = "tests/polling_performance_tests.rs"]
mod performance_tests;

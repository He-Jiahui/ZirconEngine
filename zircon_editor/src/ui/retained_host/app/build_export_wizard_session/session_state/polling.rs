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
        let profile_names = self.sessions.keys().cloned().collect::<Vec<_>>();
        let mut updates = Vec::new();
        for profile_name in profile_names {
            let before = self
                .sessions
                .get(profile_name.as_str())
                .map(|session| session.view_model().snapshot().clone());
            let result = self
                .session_mut(profile_name.as_str())
                .and_then(|session| session.handle_request(ExportWizardPanelRequest::Poll));
            let changed = match &result {
                Ok(update) => {
                    update.events_drained > 0 || before.as_ref() != Some(&update.snapshot)
                }
                Err(_) => true,
            };
            if changed {
                updates.push((profile_name, result));
            }
        }
        updates
    }
}

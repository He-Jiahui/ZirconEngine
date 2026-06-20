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
                    update.events_drained > 0
                        || before.as_ref() != Some(&update.snapshot)
                        || self
                            .last_updates
                            .get(profile_name.as_str())
                            .is_some_and(|previous| previous != update)
                }
                Err(_) => true,
            };
            if changed {
                if let Ok(update) = &result {
                    self.last_updates
                        .insert(profile_name.clone(), update.clone());
                }
                updates.push((profile_name, result));
            }
        }
        updates
    }
}

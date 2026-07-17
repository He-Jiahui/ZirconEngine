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
        updates
    }
}

#[cfg(test)]
mod performance_tests {
    #[test]
    fn polling_streams_mutable_sessions_and_skips_terminal_snapshots() {
        let source = include_str!("polling.rs");
        let production = source.split("#[cfg(test)]").next().unwrap_or(source);

        assert!(production.contains("for (profile_name, session) in &mut self.sessions"));
        assert!(production.contains("if session.view_model().snapshot().is_terminal()"));
        assert!(!production.contains("self.sessions.keys().cloned().collect"));
    }
}

use crate::ui::host::{
    ExportWizardPanelSession, ExportWizardPanelSessionError, ExportWizardPanelViewModel,
};

use super::super::surface_actions::export_wizard_job_id;
use super::DesktopExportWizardSessions;

impl DesktopExportWizardSessions {
    pub(in crate::ui::retained_host::app) fn view_model(
        &self,
        profile_name: &str,
    ) -> Option<&ExportWizardPanelViewModel> {
        self.sessions
            .get(profile_name)
            .map(ExportWizardPanelSession::view_model)
    }

    pub(in crate::ui::retained_host::app::build_export_wizard_session::session_state) fn session_mut(
        &mut self,
        profile_name: &str,
    ) -> Result<&mut ExportWizardPanelSession, ExportWizardPanelSessionError> {
        self.sessions.get_mut(profile_name).ok_or_else(|| {
            ExportWizardPanelSessionError::NoActiveJob {
                job_id: export_wizard_job_id(profile_name),
            }
        })
    }
}

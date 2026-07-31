use crate::ui::host::{
    ExportWizardPanelAction, ExportWizardPanelRequest, ExportWizardPanelSessionError,
    ExportWizardPanelUpdate, ExportWizardPipelineOptions, ProcessCommandRunner,
};

use super::DesktopExportWizardSessions;
use crate::ui::host::ExportWizardPanelSession;

use super::super::surface_actions::{export_wizard_job_id, required_options};

impl DesktopExportWizardSessions {
    pub(in crate::ui::retained_host::app) fn dispatch_profile_action(
        &mut self,
        profile_name: &str,
        action: ExportWizardPanelAction,
        options: Option<ExportWizardPipelineOptions>,
    ) -> Result<ExportWizardPanelUpdate, ExportWizardPanelSessionError> {
        self.dispatch_profile_action_with_runner(
            profile_name,
            action,
            options,
            ProcessCommandRunner::new(self.jobs.clone()),
        )
    }

    pub(in crate::ui::retained_host::app::build_export_wizard_session) fn dispatch_profile_action_with_runner(
        &mut self,
        profile_name: &str,
        action: ExportWizardPanelAction,
        options: Option<ExportWizardPipelineOptions>,
        start_runner: impl crate::ui::host::ExportWizardCommandRunner + Send + 'static,
    ) -> Result<ExportWizardPanelUpdate, ExportWizardPanelSessionError> {
        let update = match action {
            ExportWizardPanelAction::GeneratePlan => {
                let options = required_options(action, options)?;
                self.regenerate_profile_plan(profile_name, options)?
            }
            ExportWizardPanelAction::Start => {
                let options = required_options(action, options)?;
                self.regenerate_profile_plan(profile_name, options)?;
                self.session_mut(profile_name)?
                    .handle_start_request_with_runner(start_runner)?
            }
            ExportWizardPanelAction::Cancel => self
                .session_mut(profile_name)?
                .handle_request(ExportWizardPanelRequest::Cancel)?,
            ExportWizardPanelAction::Poll => self
                .session_mut(profile_name)?
                .handle_request(ExportWizardPanelRequest::Poll)?,
        };
        self.invalidate_projection_overlay();
        Ok(update)
    }

    fn regenerate_profile_plan(
        &mut self,
        profile_name: &str,
        options: ExportWizardPipelineOptions,
    ) -> Result<ExportWizardPanelUpdate, ExportWizardPanelSessionError> {
        let job_id = export_wizard_job_id(profile_name);
        let jobs = self.jobs.clone();
        let session = self
            .sessions
            .entry(profile_name.to_string())
            .or_insert_with(|| {
                ExportWizardPanelSession::from_options(jobs, job_id.clone(), options.clone())
            });
        session.handle_request(ExportWizardPanelRequest::generate_plan(job_id, options))
    }
}

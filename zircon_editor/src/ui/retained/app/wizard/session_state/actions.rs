use crate::ui::host::{
    ExportWizardPanelAction, ExportWizardPanelRequest, ExportWizardPanelSessionError,
    ExportWizardPanelUpdate, ExportWizardPipelineOptions, ProcessCommandRunner,
};

use super::DesktopExportWizardSessions;
use crate::ui::host::ExportWizardPanelSession;

use super::super::surface_actions::{export_wizard_job_id, required_options};
use zircon_runtime_interface::ui::dispatch::UiWindowId;

impl DesktopExportWizardSessions {
    pub(in crate::ui::retained_host::app) fn dispatch_profile_action(
        &mut self,
        profile_name: &str,
        action: ExportWizardPanelAction,
        options: Option<ExportWizardPipelineOptions>,
        window_id: UiWindowId,
    ) -> Result<ExportWizardPanelUpdate, ExportWizardPanelSessionError> {
        self.dispatch_profile_action_with_runner(
            profile_name,
            action,
            options,
            window_id,
            ProcessCommandRunner::new(self.jobs.clone()),
        )
    }

    pub(in crate::ui::retained_host::app::build_export_wizard_session) fn dispatch_profile_action_with_runner(
        &mut self,
        profile_name: &str,
        action: ExportWizardPanelAction,
        options: Option<ExportWizardPipelineOptions>,
        window_id: UiWindowId,
        start_runner: impl crate::ui::host::ExportWizardCommandRunner + Send + 'static,
    ) -> Result<ExportWizardPanelUpdate, ExportWizardPanelSessionError> {
        let update = match action {
            ExportWizardPanelAction::GeneratePlan => {
                let options = required_options(action, options)?;
                self.regenerate_profile_plan(profile_name, options, window_id.clone())?
            }
            ExportWizardPanelAction::Start => {
                let options = required_options(action, options)?;
                self.regenerate_profile_plan(profile_name, options, window_id.clone())?;
                self.session_mut(profile_name)?
                    .handle_start_request_with_runner(start_runner)?
            }
            ExportWizardPanelAction::Cancel => {
                let session = self.session_mut(profile_name)?;
                session.ensure_tool_window(&window_id)?;
                session.handle_request(ExportWizardPanelRequest::Cancel)?
            }
            ExportWizardPanelAction::Poll => {
                let session = self.session_mut(profile_name)?;
                session.ensure_tool_window(&window_id)?;
                session.handle_request(ExportWizardPanelRequest::Poll)?
            }
        };
        self.invalidate_projection_overlay();
        Ok(update)
    }

    fn regenerate_profile_plan(
        &mut self,
        profile_name: &str,
        options: ExportWizardPipelineOptions,
        window_id: UiWindowId,
    ) -> Result<ExportWizardPanelUpdate, ExportWizardPanelSessionError> {
        let job_id = export_wizard_job_id(profile_name);
        let jobs = self.jobs.clone();
        let tools = self.tools.clone();
        let requested_window = window_id.clone();
        let session = self
            .sessions
            .entry(profile_name.to_string())
            .or_insert_with(|| {
                ExportWizardPanelSession::from_options_with_tools(
                    jobs,
                    job_id.clone(),
                    options.clone(),
                    tools.clone(),
                    window_id,
                )
            });
        session.ensure_tool_window(&requested_window)?;
        session.handle_request(ExportWizardPanelRequest::generate_plan(job_id, options))
    }
}

use std::collections::BTreeMap;

use crate::ui::host::{
    ExportWizardPanelAction, ExportWizardPanelRequest, ExportWizardPanelSession,
    ExportWizardPanelSessionError, ExportWizardPanelUpdate, ExportWizardPanelViewModel,
    ExportWizardPipelineOptions, ProcessCommandRunner,
};

use surface_actions::{export_wizard_job_id, required_options};

mod host_actions;
mod options;
mod surface_actions;

// Owns retained app export-wizard state by profile so the pane projection can
// refresh from host state instead of rebuilding a synthetic view model each frame.
#[derive(Default)]
pub(super) struct DesktopExportWizardSessions {
    sessions: BTreeMap<String, ExportWizardPanelSession>,
    last_updates: BTreeMap<String, ExportWizardPanelUpdate>,
}

impl DesktopExportWizardSessions {
    pub(super) fn view_model(&self, profile_name: &str) -> Option<&ExportWizardPanelViewModel> {
        self.sessions
            .get(profile_name)
            .map(ExportWizardPanelSession::view_model)
    }

    pub(super) fn dispatch_profile_action(
        &mut self,
        profile_name: &str,
        action: ExportWizardPanelAction,
        options: Option<ExportWizardPipelineOptions>,
    ) -> Result<ExportWizardPanelUpdate, ExportWizardPanelSessionError> {
        self.dispatch_profile_action_with_runner(
            profile_name,
            action,
            options,
            ProcessCommandRunner,
        )
    }

    fn dispatch_profile_action_with_runner(
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
        self.last_updates
            .insert(profile_name.to_string(), update.clone());
        Ok(update)
    }

    fn regenerate_profile_plan(
        &mut self,
        profile_name: &str,
        options: ExportWizardPipelineOptions,
    ) -> Result<ExportWizardPanelUpdate, ExportWizardPanelSessionError> {
        let job_id = export_wizard_job_id(profile_name);
        let session = self
            .sessions
            .entry(profile_name.to_string())
            .or_insert_with(|| {
                ExportWizardPanelSession::from_options(job_id.clone(), options.clone())
            });
        session.handle_request(ExportWizardPanelRequest::generate_plan(job_id, options))
    }

    pub(super) fn poll_all(
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

    fn session_mut(
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

#[cfg(test)]
mod tests;

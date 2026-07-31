use crate::ui::host::{
    DESKTOP_EXPORT_CANCEL_BUTTON, DESKTOP_EXPORT_GENERATE_PLAN_BUTTON, DESKTOP_EXPORT_START_BUTTON,
    ExportWizardPanelAction, ExportWizardPanelSessionError, ExportWizardPanelUpdate,
    ExportWizardPipelineOptions,
};

use super::super::build_export_actions;

const EXPORT_WIZARD_JOB_ID_PREFIX: &str = "workbench.build_export_desktop";

#[derive(Debug, PartialEq, Eq)]
pub(super) struct BuildExportWizardSurfaceAction<'a> {
    pub(super) profile_name: &'a str,
    pub(super) action: ExportWizardPanelAction,
}

pub(super) fn build_export_wizard_surface_action<'a>(
    control_id: &str,
    action_id: &'a str,
) -> Option<BuildExportWizardSurfaceAction<'a>> {
    let action = match build_export_actions::parse_build_export_action(action_id)? {
        build_export_actions::BuildExportAction::GeneratePlan { profile_name }
            if control_id == DESKTOP_EXPORT_GENERATE_PLAN_BUTTON =>
        {
            BuildExportWizardSurfaceAction {
                profile_name,
                action: ExportWizardPanelAction::GeneratePlan,
            }
        }
        build_export_actions::BuildExportAction::Execute { profile_name }
            if control_id == DESKTOP_EXPORT_START_BUTTON =>
        {
            BuildExportWizardSurfaceAction {
                profile_name,
                action: ExportWizardPanelAction::Start,
            }
        }
        build_export_actions::BuildExportAction::Cancel { profile_name }
            if control_id == DESKTOP_EXPORT_CANCEL_BUTTON =>
        {
            BuildExportWizardSurfaceAction {
                profile_name,
                action: ExportWizardPanelAction::Cancel,
            }
        }
        _ => return None,
    };
    Some(action)
}

pub(super) fn required_options(
    action: ExportWizardPanelAction,
    options: Option<ExportWizardPipelineOptions>,
) -> Result<ExportWizardPipelineOptions, ExportWizardPanelSessionError> {
    options.ok_or(ExportWizardPanelSessionError::ActionDisabled {
        action,
        reason: "pipeline options are required",
    })
}

pub(super) fn export_wizard_job_id(profile_name: &str) -> String {
    format!("{EXPORT_WIZARD_JOB_ID_PREFIX}.{profile_name}")
}

pub(super) fn export_wizard_status_message(
    profile_name: &str,
    update: &ExportWizardPanelUpdate,
) -> String {
    match update.action {
        ExportWizardPanelAction::GeneratePlan => {
            format!("Desktop export wizard plan for {profile_name} refreshed")
        }
        ExportWizardPanelAction::Start => {
            format!("Desktop export wizard {profile_name} started")
        }
        ExportWizardPanelAction::Cancel => {
            format!("Desktop export wizard {profile_name} cancel requested")
        }
        ExportWizardPanelAction::Poll => {
            format!(
                "Desktop export wizard {profile_name} {:?}",
                update.snapshot.status
            )
        }
    }
}

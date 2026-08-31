use super::super::*;
use std::sync::OnceLock;

mod draft;
mod project;
mod recent;
mod startup_views;

use zircon_runtime_interface::project::{
    ProjectActivationOperationId, ProjectActivationOperationIdGenerator, ProjectLaunchInstanceId,
};

static PROJECT_LAUNCH_OPERATION_IDS: OnceLock<ProjectActivationOperationIdGenerator> =
    OnceLock::new();

pub(super) fn next_project_launch_operation_id() -> Result<ProjectActivationOperationId, String> {
    PROJECT_LAUNCH_OPERATION_IDS
        .get_or_init(|| ProjectActivationOperationIdGenerator::new(ProjectLaunchInstanceId::new()))
        .allocate()
        .ok_or_else(|| "project launch operation sequence is exhausted".to_string())
}

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app) fn handle_welcome_surface_event(
        &mut self,
        event: WelcomeHostEvent,
    ) {
        match event {
            WelcomeHostEvent::SetProjectName { value } => {
                self.update_welcome_project_name(value.as_str());
            }
            WelcomeHostEvent::SetLocation { value } => {
                self.update_welcome_location(value.as_str());
            }
            WelcomeHostEvent::CreateProject => self.create_project_from_welcome(),
            WelcomeHostEvent::OpenExistingProject => self.open_existing_project_from_welcome(),
            WelcomeHostEvent::OpenRecentProject { path } => {
                self.open_recent_project(path.as_str());
            }
            WelcomeHostEvent::SafeRecentProject { path } => {
                self.safe_recent_project(path.as_str());
            }
            WelcomeHostEvent::RecoverRecentProject { path } => {
                self.recover_recent_project(path.as_str());
            }
            WelcomeHostEvent::RemoveRecentProject { path } => {
                self.remove_recent_project(path.as_str());
            }
            WelcomeHostEvent::OpenStartupWorkbench => {
                self.open_startup_workbench();
            }
            WelcomeHostEvent::OpenStartupDemo => {
                self.open_startup_view(
                    "editor.ui_component_showcase",
                    "Opened UI Component Showcase",
                );
            }
            WelcomeHostEvent::OpenStartupAssetWindow => {
                self.open_startup_view(
                    "editor.asset_browser_window",
                    "Opened asset browser window",
                );
            }
            WelcomeHostEvent::OpenStartupUILayoutEditor => {
                self.open_startup_view(
                    "editor.ui_asset_editor_window",
                    "Opened UI layout editor window",
                );
            }
        }
    }
}

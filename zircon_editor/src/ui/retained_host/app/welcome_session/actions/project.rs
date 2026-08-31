use super::super::super::*;
use super::next_project_launch_operation_id;
use zircon_runtime_interface::project::{
    ProjectLaunchIntent, ProjectLaunchProfile, ProjectLaunchSource,
};

impl RetainedEditorHost {
    pub(super) fn create_project_from_welcome(&mut self) {
        let draft = self.startup_session.draft.clone();
        let result = next_project_launch_operation_id()
            .and_then(|operation_id| {
                ProjectLaunchIntent::create_project(
                    operation_id,
                    ProjectLaunchSource::Welcome,
                    ProjectLaunchProfile::Normal,
                    draft.project_name,
                    draft.location,
                    draft.template.pack_id(),
                )
                .map_err(|error| error.to_string())
            })
            .and_then(|intent| {
                self.editor_manager
                    .execute_project_launch_intent(intent)
                    .map_err(|error| error.to_string())
            })
            .map_err(|error| error.to_string())
            .and_then(|session| self.apply_startup_session(session));
        match result {
            Ok(()) => {}
            Err(error) => {
                self.startup_session.status_message = error.clone();
                self.refresh_welcome_snapshot();
                self.set_status_line(error);
            }
        }
    }

    pub(super) fn open_existing_project_from_welcome(&mut self) {
        let result = self
            .startup_session
            .draft
            .project_root()
            .map_err(|error| error.to_string())
            .and_then(|project_root| {
                next_project_launch_operation_id().and_then(|operation_id| {
                    ProjectLaunchIntent::open_existing(
                        operation_id,
                        ProjectLaunchSource::Welcome,
                        ProjectLaunchProfile::Normal,
                        project_root,
                    )
                    .map_err(|error| error.to_string())
                })
            })
            .and_then(|intent| {
                self.editor_manager
                    .execute_project_launch_intent(intent)
                    .map_err(|error| error.to_string())
            })
            .and_then(|session| self.apply_startup_session(session));
        if let Err(error) = result {
            self.startup_session.status_message = error.clone();
            self.refresh_welcome_snapshot();
            self.set_status_line(error);
        }
    }
}

use super::super::super::*;
use super::next_project_launch_operation_id;
use zircon_runtime_interface::project::{
    ProjectLaunchIntent, ProjectLaunchProfile, ProjectLaunchSource,
};

impl RetainedEditorHost {
    pub(super) fn open_recent_project(&mut self, path: &str) {
        self.launch_recent_project(path, ProjectLaunchProfile::Normal);
    }

    pub(super) fn safe_recent_project(&mut self, path: &str) {
        self.launch_recent_project(path, ProjectLaunchProfile::Safe);
    }

    pub(super) fn recover_recent_project(&mut self, path: &str) {
        self.launch_recent_project(path, ProjectLaunchProfile::Recovery);
    }

    fn launch_recent_project(&mut self, path: &str, profile: ProjectLaunchProfile) {
        let result = next_project_launch_operation_id()
            .and_then(|operation_id| {
                ProjectLaunchIntent::open_existing(
                    operation_id,
                    ProjectLaunchSource::Recent,
                    profile,
                    path,
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
        if let Err(error) = result {
            self.startup_session.status_message = error.clone();
            if let Ok(recent_projects) = self.editor_manager.recent_projects_snapshot() {
                self.startup_session.recent_projects = recent_projects;
            }
            self.refresh_welcome_snapshot();
            self.set_status_line(error);
        }
    }

    pub(super) fn remove_recent_project(&mut self, path: &str) {
        match self
            .editor_manager
            .forget_recent_project(path)
            .map_err(|error| error.to_string())
            .and_then(|_| {
                self.editor_manager
                    .recent_projects_snapshot()
                    .map_err(|error| error.to_string())
            }) {
            Ok(recent_projects) => {
                self.startup_session.recent_projects = recent_projects;
                self.startup_session.status_message = format!("Removed recent project {path}");
                self.refresh_welcome_snapshot();
                self.set_status_line(format!("Removed recent project {path}"));
            }
            Err(error) => {
                self.startup_session.status_message = error.clone();
                self.refresh_welcome_snapshot();
                self.set_status_line(error);
            }
        }
    }
}

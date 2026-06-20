use super::super::super::*;

impl RetainedEditorHost {
    pub(super) fn open_recent_project(&mut self, path: &str) {
        let result = self
            .editor_manager
            .open_project_and_remember(path)
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

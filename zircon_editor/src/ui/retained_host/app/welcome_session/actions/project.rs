use super::super::super::*;

impl RetainedEditorHost {
    pub(super) fn create_project_from_welcome(&mut self) {
        match self
            .editor_manager
            .create_project_and_open(self.startup_session.draft.clone())
            .map_err(|error| error.to_string())
            .and_then(|session| self.apply_startup_session(session))
        {
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
            .validate_for_open_existing()
            .map_err(|error| error.to_string())
            .and_then(|root| {
                self.editor_manager
                    .open_project_and_remember(root)
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

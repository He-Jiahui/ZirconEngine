use super::super::super::*;

fn welcome_session_after_project_close(
    mut session: EditorStartupSessionDocument,
    closed_root: Option<&std::path::Path>,
) -> EditorStartupSessionDocument {
    session.mode = EditorSessionMode::Welcome;
    session.project = None;
    session.open_builtin_view = None;
    session.status_message = closed_root.map_or_else(
        || "Project was already closed; restored the welcome workspace.".to_string(),
        |root| format!("Closed project {}", root.display()),
    );
    session
}

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app) fn close_project_from_workbench(
        &mut self,
    ) -> Result<(), String> {
        let closed_root = self
            .editor_manager
            .close_project()
            .map_err(|error| error.to_string())?;

        let welcome_session = welcome_session_after_project_close(
            self.startup_session.clone(),
            closed_root.as_deref(),
        );
        self.apply_startup_session(welcome_session)
    }

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
        let result = crate::core::project::ProjectAuthority::default()
            .probe_draft(&self.startup_session.draft)
            .map_err(|error| error.to_string())
            .and_then(|opened| {
                self.editor_manager
                    .open_project_and_remember(opened.root())
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

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::ui::workbench::startup::{EditorSessionMode, EditorStartupSessionDocument};

    use super::welcome_session_after_project_close;

    #[test]
    fn successful_close_returns_to_welcome_without_retaining_project_navigation() {
        let mut session = EditorStartupSessionDocument::default();
        session.mode = EditorSessionMode::Project;
        session.open_builtin_view = Some("editor.scene".to_string());

        let welcome =
            welcome_session_after_project_close(session, Some(Path::new("C:/projects/forest")));

        assert_eq!(welcome.mode, EditorSessionMode::Welcome);
        assert!(welcome.project.is_none());
        assert!(welcome.open_builtin_view.is_none());
        assert_eq!(welcome.status_message, "Closed project C:/projects/forest");
    }

    #[test]
    fn retry_after_committed_runtime_close_still_repairs_the_welcome_surface() {
        let mut session = EditorStartupSessionDocument::default();
        session.mode = EditorSessionMode::Project;
        session.open_builtin_view = Some("editor.asset_browser".to_string());

        let welcome = welcome_session_after_project_close(session, None);

        assert_eq!(welcome.mode, EditorSessionMode::Welcome);
        assert!(welcome.project.is_none());
        assert!(welcome.open_builtin_view.is_none());
        assert_eq!(
            welcome.status_message,
            "Project was already closed; restored the welcome workspace."
        );
    }
}

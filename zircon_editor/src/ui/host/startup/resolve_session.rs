use crate::ui::workbench::startup::{
    display_project_path, EditorSessionMode, EditorStartupSessionDocument, NewProjectDraft,
    RecentProjectValidation,
};

use super::super::editor_error::EditorError;
use super::super::editor_ui_host::EditorUiHost;

impl EditorUiHost {
    pub fn resolve_startup_session(&self) -> Result<EditorStartupSessionDocument, EditorError> {
        let stored = self.load_startup_session()?;
        let recent_projects =
            stored.recent_projects_with_validation(|path| self.validate_recent_project(path));
        let mut session = EditorStartupSessionDocument {
            recent_projects,
            draft: NewProjectDraft::renderable_empty_default(),
            ..EditorStartupSessionDocument::default()
        };

        if let Some(path) = stored.last_project_path.as_deref() {
            if self.validate_recent_project(path) == RecentProjectValidation::Valid {
                let document = self.open_project(path)?;
                self.dismiss_welcome_page()?;
                session.mode = EditorSessionMode::Project;
                session.project = Some(document);
                session.status_message = format!("Reopened {}", display_project_path(path));
                return Ok(session);
            }

            self.show_welcome_page()?;
            session.mode = EditorSessionMode::Welcome;
            session.status_message = format!(
                "Last project is unavailable: {}. Choose a recent project or create a new one.",
                display_project_path(path)
            );
            return Ok(session);
        }

        self.show_welcome_page()?;
        Ok(session)
    }
}

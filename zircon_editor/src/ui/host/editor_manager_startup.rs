use std::path::Path;

use crate::core::project::{NewProjectDraft, RecentProjectEntry};
use crate::ui::workbench::startup::EditorStartupSessionDocument;

use super::editor_error::EditorError;
use super::editor_manager::EditorManager;

impl EditorManager {
    pub fn resolve_startup_session(&self) -> Result<EditorStartupSessionDocument, EditorError> {
        self.host
            .resolve_startup_session_with_project_open(|path| self.open_project(path))
    }

    pub fn open_project_and_remember(
        &self,
        path: impl AsRef<Path>,
    ) -> Result<EditorStartupSessionDocument, EditorError> {
        self.open_project_and_remember_with_session(path)
    }

    pub fn create_project_and_open(
        &self,
        draft: NewProjectDraft,
    ) -> Result<EditorStartupSessionDocument, EditorError> {
        self.create_project_and_open_with_session(draft)
    }

    pub fn recent_projects_snapshot(&self) -> Result<Vec<RecentProjectEntry>, EditorError> {
        self.host.recent_projects_snapshot()
    }

    pub fn forget_recent_project(&self, path: impl AsRef<Path>) -> Result<(), EditorError> {
        self.host.forget_recent_project(path)
    }

    pub fn update_recent_project(&self, path: impl AsRef<Path>) -> Result<(), EditorError> {
        self.host.update_recent_project(path)
    }

    pub(crate) fn show_welcome_page(&self) -> Result<(), EditorError> {
        self.host.show_welcome_page()
    }

    pub(crate) fn dismiss_welcome_page(&self) -> Result<(), EditorError> {
        self.host.dismiss_welcome_page()
    }
}

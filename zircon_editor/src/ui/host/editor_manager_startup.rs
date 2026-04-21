use std::path::Path;

use crate::ui::workbench::startup::{
    EditorStartupSessionDocument, NewProjectDraft, RecentProjectEntry,
};

use super::editor_error::EditorError;
use super::editor_manager::EditorManager;

impl EditorManager {
    pub fn resolve_startup_session(&self) -> Result<EditorStartupSessionDocument, EditorError> {
        self.host.resolve_startup_session()
    }

    pub fn open_project_and_remember(
        &self,
        path: impl AsRef<Path>,
    ) -> Result<EditorStartupSessionDocument, EditorError> {
        self.host.open_project_and_remember(path)
    }

    pub fn create_project_and_open(
        &self,
        draft: NewProjectDraft,
    ) -> Result<EditorStartupSessionDocument, EditorError> {
        self.host.create_project_and_open(draft)
    }

    pub fn recent_projects_snapshot(&self) -> Result<Vec<RecentProjectEntry>, EditorError> {
        self.host.recent_projects_snapshot()
    }

    pub fn forget_recent_project(&self, path: impl AsRef<Path>) -> Result<(), EditorError> {
        self.host.forget_recent_project(path)
    }

    pub fn update_recent_project(
        &self,
        path: impl AsRef<Path>,
        display_name: &str,
    ) -> Result<(), EditorError> {
        self.host.update_recent_project(path, display_name)
    }

    pub(crate) fn show_welcome_page(&self) -> Result<(), EditorError> {
        self.host.show_welcome_page()
    }

    pub(crate) fn dismiss_welcome_page(&self) -> Result<(), EditorError> {
        self.host.dismiss_welcome_page()
    }
}

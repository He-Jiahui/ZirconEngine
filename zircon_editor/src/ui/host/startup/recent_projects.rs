use std::path::Path;

use crate::ui::workbench::startup::{now_unix_ms, RecentProjectEntry};

use super::super::editor_error::EditorError;
use super::super::editor_ui_host::EditorUiHost;
use super::canonical_project_root::canonical_project_root;

impl EditorUiHost {
    pub fn recent_projects_snapshot(&self) -> Result<Vec<RecentProjectEntry>, EditorError> {
        Ok(self
            .load_startup_session()?
            .recent_projects_with_validation(|path| self.validate_recent_project(path)))
    }

    pub fn forget_recent_project(&self, path: impl AsRef<Path>) -> Result<(), EditorError> {
        let mut stored = self.load_startup_session()?;
        stored.forget_recent_project(path.as_ref().to_string_lossy().as_ref());
        self.save_startup_session(&stored)
    }

    pub fn update_recent_project(
        &self,
        path: impl AsRef<Path>,
        display_name: &str,
    ) -> Result<(), EditorError> {
        let path = canonical_project_root(path.as_ref())
            .map_err(|error| EditorError::Project(error.to_string()))?;
        let mut stored = self.load_startup_session()?;
        stored.update_recent_project(path.to_string_lossy().as_ref(), display_name, now_unix_ms());
        self.save_startup_session(&stored)
    }
}

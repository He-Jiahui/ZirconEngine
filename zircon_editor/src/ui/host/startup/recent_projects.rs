use std::path::Path;

use crate::core::project::{ProjectAuthority, RecentProjectEntry};
use crate::ui::workbench::startup::now_unix_ms;

use super::super::editor_error::EditorError;
use super::super::editor_ui_host::EditorUiHost;

impl EditorUiHost {
    pub fn recent_projects_snapshot(&self) -> Result<Vec<RecentProjectEntry>, EditorError> {
        Ok(ProjectAuthority::default()
            .recent_projects_with_validation(&self.load_startup_session()?, |path| {
                ProjectAuthority::default().validate_recent_project(path)
            }))
    }

    pub fn forget_recent_project(&self, path: impl AsRef<Path>) -> Result<(), EditorError> {
        let mut stored = self.load_startup_session()?;
        ProjectAuthority::default()
            .forget_recent_project(&mut stored, path.as_ref().to_string_lossy().as_ref());
        self.save_startup_session(&stored)
    }

    pub fn update_recent_project(&self, path: impl AsRef<Path>) -> Result<(), EditorError> {
        let opened = ProjectAuthority::default().open_project(path)?;
        let path = opened.root.to_string_lossy().into_owned();
        let mut stored = self.load_startup_session()?;
        ProjectAuthority::default().remember_recent_project(
            &mut stored,
            &path,
            opened.summary,
            now_unix_ms(),
        );
        self.save_startup_session(&stored)
    }
}

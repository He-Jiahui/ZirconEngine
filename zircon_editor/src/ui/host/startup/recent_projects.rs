use std::path::Path;

use crate::core::hub_link::{forget_recent_project, load_recent_projects, record_recent_project};
use crate::core::project::{ProjectAuthority, RecentProjectEntry};
use crate::ui::workbench::startup::now_unix_ms;

use super::super::editor_error::EditorError;
use super::super::editor_ui_host::EditorUiHost;

impl EditorUiHost {
    pub fn recent_projects_snapshot(&self) -> Result<Vec<RecentProjectEntry>, EditorError> {
        let registry = load_recent_projects().map_err(shared_recent_projects_error)?;
        let authority = ProjectAuthority::default();
        Ok(registry
            .projects
            .into_iter()
            .map(|project| {
                let path = project.path.to_string_lossy().into_owned();
                let validation = authority.validate_recent_project(&path);
                RecentProjectEntry::from_shared(project, validation)
            })
            .collect())
    }

    pub fn forget_recent_project(&self, path: impl AsRef<Path>) -> Result<(), EditorError> {
        forget_recent_project(path.as_ref()).map_err(shared_recent_projects_error)?;
        Ok(())
    }

    pub fn update_recent_project(&self, path: impl AsRef<Path>) -> Result<(), EditorError> {
        let opened = ProjectAuthority::default().probe_project(path)?;
        record_recent_project(opened.root(), opened.summary().clone(), now_unix_ms())
            .map_err(shared_recent_projects_error)?;
        Ok(())
    }
}

fn shared_recent_projects_error(
    error: crate::core::hub_link::HubRecentProjectsStoreError,
) -> EditorError {
    EditorError::Project(format!("shared recent-project registry failed: {error}"))
}

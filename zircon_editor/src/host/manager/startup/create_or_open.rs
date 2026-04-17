use crate::project::EditorProjectDocument;
use crate::workbench::startup::{EditorSessionMode, EditorStartupSessionDocument, NewProjectDraft};

use super::super::editor_error::EditorError;
use super::super::editor_manager::EditorManager;

impl EditorManager {
    pub fn open_project_and_remember(
        &self,
        path: impl AsRef<std::path::Path>,
    ) -> Result<EditorStartupSessionDocument, EditorError> {
        let document = self.open_project(&path)?;
        self.update_recent_project(&document.root_path, document.manifest.name.as_str())?;
        self.dismiss_welcome_page()?;

        Ok(EditorStartupSessionDocument {
            mode: EditorSessionMode::Project,
            project: Some(document),
            recent_projects: self.recent_projects_snapshot()?,
            draft: NewProjectDraft::renderable_empty_default(),
            status_message: "Project opened".to_string(),
        })
    }

    pub fn create_project_and_open(
        &self,
        draft: NewProjectDraft,
    ) -> Result<EditorStartupSessionDocument, EditorError> {
        let root = EditorProjectDocument::create_renderable_template(&draft)
            .map_err(|error| EditorError::Project(error.to_string()))?;
        self.open_project_and_remember(root)
    }
}

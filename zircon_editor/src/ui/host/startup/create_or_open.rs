use crate::core::project::{NewProjectDraft, ProjectAuthority};
use crate::ui::workbench::project::EditorProjectDocument;
use crate::ui::workbench::startup::{EditorSessionMode, EditorStartupSessionDocument};

use super::super::editor_error::EditorError;
use super::super::editor_ui_host::EditorUiHost;

impl EditorUiHost {
    pub fn open_project_and_remember(
        &self,
        path: impl AsRef<std::path::Path>,
    ) -> Result<EditorStartupSessionDocument, EditorError> {
        let document = self.open_project(&path)?;
        let status_message = project_open_status_message(&document);
        self.remember_opened_project(&document.root_path, document.manifest.summary())?;
        self.dismiss_welcome_page()?;

        Ok(EditorStartupSessionDocument {
            mode: EditorSessionMode::Project,
            project: Some(document),
            open_builtin_view: None,
            recent_projects: self.recent_projects_snapshot()?,
            draft: NewProjectDraft::renderable_empty_default(),
            status_message,
        })
    }

    pub fn create_project_and_open(
        &self,
        draft: NewProjectDraft,
    ) -> Result<EditorStartupSessionDocument, EditorError> {
        let created = ProjectAuthority::default().create_project(&draft)?;
        self.open_project_and_remember(created.root)
    }
}

fn project_open_status_message(document: &EditorProjectDocument) -> String {
    let Some(diagnostic) = document.workspace_restore_diagnostics.first() else {
        return "Project opened".to_string();
    };
    format!(
        "Project opened with default layout; failed to restore workspace layout from {}: {}",
        diagnostic.path.display(),
        diagnostic.message
    )
}

#[cfg(test)]
mod tests {
    #[test]
    fn opened_project_is_not_reopened_just_to_update_recents() {
        let source = include_str!("create_or_open.rs");
        let reopening_call = ["self", ".update_recent_project(&document.root_path)"].concat();
        assert!(!source.contains(&reopening_call));
    }
}

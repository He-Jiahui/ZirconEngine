use crate::ui::workbench::startup::{EditorStartupSessionDocument, NewProjectDraft};

use super::super::editor_error::EditorError;
use super::super::editor_ui_host::EditorUiHost;

impl EditorUiHost {
    pub fn resolve_startup_session(&self) -> Result<EditorStartupSessionDocument, EditorError> {
        zircon_runtime::profile_scope!("editor", "startup_session", "resolve_startup_session");

        Ok(EditorStartupSessionDocument {
            open_builtin_view: Some("editor.ui_component_showcase".to_string()),
            recent_projects: Vec::new(),
            draft: {
                zircon_runtime::profile_scope!("editor", "startup_session", "build_default_draft");
                NewProjectDraft::renderable_empty_default()
            },
            status_message: "Opened UI Component Showcase".to_string(),
            ..EditorStartupSessionDocument::default()
        })
    }
}

use super::editor_session_mode::EditorSessionMode;
use super::editor_startup_session_document::EditorStartupSessionDocument;
use super::new_project_draft::NewProjectDraft;

impl Default for EditorStartupSessionDocument {
    fn default() -> Self {
        Self {
            mode: EditorSessionMode::Welcome,
            project: None,
            recent_projects: Vec::new(),
            draft: NewProjectDraft::renderable_empty_default(),
            status_message: "Open an existing project or create a renderable empty project."
                .to_string(),
        }
    }
}

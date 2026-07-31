use super::editor_session_mode::EditorSessionMode;
use crate::core::project::{NewProjectDraft, RecentProjectEntry};
use crate::ui::workbench::project::EditorProjectDocument;

#[derive(Clone, Debug, PartialEq)]
pub struct EditorStartupSessionDocument {
    pub mode: EditorSessionMode,
    pub project: Option<EditorProjectDocument>,
    pub open_builtin_view: Option<String>,
    pub recent_projects: Vec<RecentProjectEntry>,
    pub draft: NewProjectDraft,
    pub creation_validation: String,
    pub can_open_existing: bool,
    pub status_message: String,
}

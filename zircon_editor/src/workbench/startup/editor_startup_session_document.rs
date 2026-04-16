use super::editor_session_mode::EditorSessionMode;
use super::new_project_draft::NewProjectDraft;
use super::recent_project_entry::RecentProjectEntry;
use crate::workbench::project::EditorProjectDocument;

#[derive(Clone, Debug, PartialEq)]
pub struct EditorStartupSessionDocument {
    pub mode: EditorSessionMode,
    pub project: Option<EditorProjectDocument>,
    pub recent_projects: Vec<RecentProjectEntry>,
    pub draft: NewProjectDraft,
    pub status_message: String,
}

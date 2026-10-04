use super::editor_session_mode::EditorSessionMode;
use crate::core::project::{NewProjectDraft, RecentProjectEntry};
use crate::ui::workbench::project::EditorProjectDocument;

#[derive(Clone, Debug, PartialEq)]
/// chooser或已准备项目的宿主交接载荷；recent列表不自动激活项目，实际world还需提交生命周期。
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

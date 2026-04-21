use crate::ui::workbench::autolayout::ShellFrame;
use crate::ui::workbench::layout::MainPageId;
use crate::ui::workbench::view::ViewInstanceId;

use super::DocumentWorkspaceSnapshot;

#[derive(Clone, Debug)]
pub struct FloatingWindowSnapshot {
    pub window_id: MainPageId,
    pub title: String,
    pub requested_frame: ShellFrame,
    pub workspace: DocumentWorkspaceSnapshot,
    pub focused_view: Option<ViewInstanceId>,
}

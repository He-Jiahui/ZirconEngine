use crate::layout::MainPageId;
use crate::view::ViewInstanceId;

use super::DocumentWorkspaceSnapshot;

#[derive(Clone, Debug)]
pub struct FloatingWindowSnapshot {
    pub window_id: MainPageId,
    pub title: String,
    pub workspace: DocumentWorkspaceSnapshot,
    pub focused_view: Option<ViewInstanceId>,
}

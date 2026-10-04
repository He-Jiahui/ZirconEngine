use crate::ui::workbench::layout::{DocumentNodeId, SplitAxis};
use crate::ui::workbench::view::ViewInstanceId;

use super::ViewTabSnapshot;

#[derive(Clone, Debug)]
pub enum DocumentWorkspaceSnapshot {
    Split {
        node_id: DocumentNodeId,
        axis: SplitAxis,
        ratio: f32,
        first: Box<DocumentWorkspaceSnapshot>,
        second: Box<DocumentWorkspaceSnapshot>,
    },
    Tabs {
        node_id: DocumentNodeId,
        tabs: Vec<ViewTabSnapshot>,
        active_tab: Option<ViewInstanceId>,
    },
}

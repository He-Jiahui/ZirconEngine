use crate::layout::SplitAxis;
use crate::view::ViewInstanceId;

use super::ViewTabSnapshot;

#[derive(Clone, Debug)]
pub enum DocumentWorkspaceSnapshot {
    Split {
        axis: SplitAxis,
        ratio: f32,
        first: Box<DocumentWorkspaceSnapshot>,
        second: Box<DocumentWorkspaceSnapshot>,
    },
    Tabs {
        tabs: Vec<ViewTabSnapshot>,
        active_tab: Option<ViewInstanceId>,
    },
}

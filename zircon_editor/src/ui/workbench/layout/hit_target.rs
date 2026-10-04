use serde::{Deserialize, Serialize};

use super::{ActivityDrawerSlot, DockEdge, MainPageId};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
/// 布局命中目标；路径仅对应命中时的工作区树，拓扑变化后不可直接复用。
pub enum HitTarget {
    Drawer(ActivityDrawerSlot),
    Document(MainPageId, Vec<usize>),
    DocumentEdge {
        page_id: MainPageId,
        path: Vec<usize>,
        edge: DockEdge,
    },
    FloatingWindow(MainPageId, Vec<usize>),
    FloatingWindowEdge {
        window_id: MainPageId,
        path: Vec<usize>,
        edge: DockEdge,
    },
    ExclusivePage(MainPageId),
    NewFloatingWindow,
}

use serde::{Deserialize, Serialize};

use super::{ActivityDrawerSlot, DockEdge, MainPageId};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
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

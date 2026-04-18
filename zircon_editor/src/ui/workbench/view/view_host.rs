use serde::{Deserialize, Serialize};

use crate::layout::{ActivityDrawerSlot, MainPageId};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ViewHost {
    Drawer(ActivityDrawerSlot),
    Document(MainPageId, Vec<usize>),
    FloatingWindow(MainPageId, Vec<usize>),
    ExclusivePage(MainPageId),
}

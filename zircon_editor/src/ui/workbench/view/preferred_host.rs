use serde::{Deserialize, Serialize};

use crate::layout::ActivityDrawerSlot;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PreferredHost {
    Drawer(ActivityDrawerSlot),
    DocumentCenter,
    FloatingWindow,
    ExclusiveMainPage,
}

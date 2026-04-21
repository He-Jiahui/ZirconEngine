use serde::{Deserialize, Serialize};

use crate::ui::workbench::layout::ActivityDrawerSlot;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PreferredHost {
    Drawer(ActivityDrawerSlot),
    DocumentCenter,
    FloatingWindow,
    ExclusiveMainPage,
}

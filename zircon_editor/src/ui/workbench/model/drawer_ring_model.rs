use std::collections::BTreeMap;

use crate::ui::workbench::layout::ActivityDrawerSlot;
use crate::ui::workbench::snapshot::ActivityDrawerSnapshot;

#[derive(Clone, Debug)]
pub struct DrawerRingModel {
    pub visible: bool,
    pub drawers: BTreeMap<ActivityDrawerSlot, ActivityDrawerSnapshot>,
}

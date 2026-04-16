use std::collections::BTreeMap;

use crate::layout::ActivityDrawerSlot;

#[derive(Clone, Debug)]
pub struct DrawerRingModel {
    pub visible: bool,
    pub drawers: BTreeMap<ActivityDrawerSlot, crate::ActivityDrawerSnapshot>,
}

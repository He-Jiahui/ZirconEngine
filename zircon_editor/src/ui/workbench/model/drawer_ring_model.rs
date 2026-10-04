use std::collections::BTreeMap;

use crate::ui::workbench::layout::ActivityDrawerSlot;
use crate::ui::workbench::snapshot::ActivityDrawerSnapshot;

#[derive(Clone, Debug)]
/// 当前活动窗口的抽屉集合；ring可见性与单槽折叠/隐藏状态分别由解算器消费。
pub struct DrawerRingModel {
    pub visible: bool,
    pub drawers: BTreeMap<ActivityDrawerSlot, ActivityDrawerSnapshot>,
}

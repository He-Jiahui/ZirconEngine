use std::collections::BTreeMap;

use crate::ui::workbench::layout::{ActivityDrawerSlot, MainPageId};

use super::{ActivityDrawerSnapshot, FloatingWindowSnapshot, MainPageSnapshot};

#[derive(Clone, Debug)]
/// 全部主页面/浮层，加当前活动activity window的抽屉；active_main_page单独维持稳定ID。
pub struct WorkbenchSnapshot {
    pub active_main_page: MainPageId,
    pub main_pages: Vec<MainPageSnapshot>,
    pub drawers: BTreeMap<ActivityDrawerSlot, ActivityDrawerSnapshot>,
    pub floating_windows: Vec<FloatingWindowSnapshot>,
}

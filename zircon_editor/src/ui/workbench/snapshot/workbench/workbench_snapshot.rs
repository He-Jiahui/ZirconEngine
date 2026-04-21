use std::collections::BTreeMap;

use crate::ui::workbench::layout::{ActivityDrawerSlot, MainPageId};

use super::{ActivityDrawerSnapshot, FloatingWindowSnapshot, MainPageSnapshot};

#[derive(Clone, Debug)]
pub struct WorkbenchSnapshot {
    pub active_main_page: MainPageId,
    pub main_pages: Vec<MainPageSnapshot>,
    pub drawers: BTreeMap<ActivityDrawerSlot, ActivityDrawerSnapshot>,
    pub floating_windows: Vec<FloatingWindowSnapshot>,
}

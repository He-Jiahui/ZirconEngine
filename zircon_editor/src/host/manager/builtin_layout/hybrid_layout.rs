use std::collections::BTreeMap;

use crate::layout::{ActivityDrawerSlot, WorkbenchLayout};

use super::layout_drawers::{
    bottom_left_drawer, bottom_right_drawer, left_bottom_drawer, left_top_drawer,
    right_bottom_drawer, right_top_drawer,
};
use super::workbench_page::builtin_workbench_page;

pub(crate) fn builtin_hybrid_layout() -> WorkbenchLayout {
    WorkbenchLayout {
        active_main_page: crate::layout::MainPageId::workbench(),
        main_pages: vec![builtin_workbench_page()],
        drawers: BTreeMap::from([
            (ActivityDrawerSlot::LeftTop, left_top_drawer()),
            (ActivityDrawerSlot::LeftBottom, left_bottom_drawer()),
            (ActivityDrawerSlot::RightTop, right_top_drawer()),
            (ActivityDrawerSlot::RightBottom, right_bottom_drawer()),
            (ActivityDrawerSlot::BottomLeft, bottom_left_drawer()),
            (ActivityDrawerSlot::BottomRight, bottom_right_drawer()),
        ]),
        floating_windows: Vec::new(),
        region_overrides: BTreeMap::new(),
        view_overrides: BTreeMap::new(),
    }
}

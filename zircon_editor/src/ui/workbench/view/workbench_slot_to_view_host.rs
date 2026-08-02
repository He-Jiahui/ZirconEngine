use crate::core::extension::WorkbenchSlot;
use crate::ui::workbench::layout::{ActivityDrawerSlot, MainPageId};

use super::ViewHost;

pub(super) fn workbench_slot_to_view_host(workbench_slot: WorkbenchSlot) -> ViewHost {
    match workbench_slot {
        WorkbenchSlot::LeftTopDrawer => ViewHost::Drawer(ActivityDrawerSlot::LeftTop),
        WorkbenchSlot::LeftBottomDrawer => ViewHost::Drawer(ActivityDrawerSlot::LeftBottom),
        WorkbenchSlot::RightTopDrawer => ViewHost::Drawer(ActivityDrawerSlot::RightTop),
        WorkbenchSlot::RightBottomDrawer => ViewHost::Drawer(ActivityDrawerSlot::RightBottom),
        WorkbenchSlot::BottomDrawer => ViewHost::Drawer(ActivityDrawerSlot::Bottom),
        WorkbenchSlot::DocumentCenter => ViewHost::Document(MainPageId::workbench(), vec![]),
        WorkbenchSlot::FloatingWindow => {
            ViewHost::FloatingWindow(MainPageId::new("floating"), vec![])
        }
        WorkbenchSlot::ExclusiveMainPage => ViewHost::ExclusivePage(MainPageId::new("exclusive")),
    }
}

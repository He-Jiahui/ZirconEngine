use crate::ui::workbench::autolayout::ShellRegionId;
use crate::ui::workbench::layout::{ActivityDrawerMode, ActivityDrawerSlot};
use crate::ui::workbench::snapshot::{MainPageSnapshot, ViewContentKind, WorkbenchSnapshot};

use super::super::super::asset_visibility::active_workspace_tab;

fn drawer_slot_region(slot: ActivityDrawerSlot) -> ShellRegionId {
    match slot {
        ActivityDrawerSlot::LeftTop | ActivityDrawerSlot::LeftBottom => ShellRegionId::Left,
        ActivityDrawerSlot::RightTop | ActivityDrawerSlot::RightBottom => ShellRegionId::Right,
        ActivityDrawerSlot::Bottom
        | ActivityDrawerSlot::BottomLeft
        | ActivityDrawerSlot::BottomRight => ShellRegionId::Bottom,
    }
}

pub(super) fn active_drawer_region_for_kind(
    workbench: &WorkbenchSnapshot,
    kind: ViewContentKind,
) -> Option<ShellRegionId> {
    workbench
        .drawers
        .values()
        .find(|drawer| {
            drawer.visible
                && drawer.mode != ActivityDrawerMode::Collapsed
                && drawer
                    .active_tab
                    .as_ref()
                    .and_then(|active| drawer.tabs.iter().find(|tab| &tab.instance_id == active))
                    .or_else(|| drawer.tabs.first())
                    .is_some_and(|tab| tab.content_kind == kind)
        })
        .map(|drawer| drawer_slot_region(drawer.slot))
}

pub(super) fn active_main_page_matches_kind(
    workbench: &WorkbenchSnapshot,
    kind: ViewContentKind,
) -> bool {
    let Some(page) = workbench.main_pages.iter().find(|page| match page {
        MainPageSnapshot::Workbench { id, .. } | MainPageSnapshot::Exclusive { id, .. } => {
            id == &workbench.active_main_page
        }
    }) else {
        return false;
    };

    match page {
        MainPageSnapshot::Workbench { workspace, .. } => {
            active_workspace_tab(workspace).is_some_and(|tab| tab.content_kind == kind)
        }
        MainPageSnapshot::Exclusive { view, .. } => view.content_kind == kind,
    }
}

pub(super) fn active_workbench_main_page_matches_kind(
    workbench: &WorkbenchSnapshot,
    kind: ViewContentKind,
) -> bool {
    let Some(MainPageSnapshot::Workbench { workspace, .. }) =
        workbench.main_pages.iter().find(|page| match page {
            MainPageSnapshot::Workbench { id, .. } | MainPageSnapshot::Exclusive { id, .. } => {
                id == &workbench.active_main_page
            }
        })
    else {
        return false;
    };

    active_workspace_tab(workspace).is_some_and(|tab| tab.content_kind == kind)
}

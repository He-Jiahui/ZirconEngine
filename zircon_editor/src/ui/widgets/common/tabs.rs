use slint::SharedString;

use crate::ui::layouts::windows::workbench_host_window::TabData;
use crate::ui::workbench::layout::ActivityDrawerSlot;
use crate::ui::workbench::model::{DocumentTabModel, HostPageTabModel, WorkbenchViewModel};

pub(crate) fn host_tab_data(
    page: &HostPageTabModel,
    active_page: &crate::ui::workbench::layout::MainPageId,
) -> TabData {
    TabData {
        id: page.id.0.clone().into(),
        slot: SharedString::default(),
        title: page.title.clone().into(),
        icon_key: host_tab_icon(page).into(),
        active: &page.id == active_page,
        closeable: page.closeable,
    }
}

pub(crate) fn document_tab_data(tab: &DocumentTabModel) -> TabData {
    TabData {
        id: tab.instance_id.0.clone().into(),
        slot: SharedString::default(),
        title: tab.title.clone().into(),
        icon_key: tab.icon_key.clone().into(),
        active: tab.active,
        closeable: tab.closeable,
    }
}

pub(crate) fn collect_tabs(
    model: &WorkbenchViewModel,
    slots: &[ActivityDrawerSlot],
) -> Vec<TabData> {
    slots
        .iter()
        .filter_map(|slot| model.tool_windows.get(slot))
        .flat_map(|stack| {
            stack.tabs.iter().map(move |tab| TabData {
                id: tab.instance_id.0.clone().into(),
                slot: drawer_slot_key(stack.slot).into(),
                title: tab.title.clone().into(),
                icon_key: tab.icon_key.clone().into(),
                active: tab.active,
                closeable: tab.closeable,
            })
        })
        .collect()
}

pub(crate) fn side_expanded(model: &WorkbenchViewModel, slots: &[ActivityDrawerSlot]) -> bool {
    slots
        .iter()
        .filter_map(|slot| model.tool_windows.get(slot))
        .any(|stack| {
            stack.visible
                && !stack.tabs.is_empty()
                && stack.mode != crate::ui::workbench::layout::ActivityDrawerMode::Collapsed
        })
}

pub(crate) fn drawer_slot_key(slot: ActivityDrawerSlot) -> &'static str {
    match slot {
        ActivityDrawerSlot::LeftTop => "left_top",
        ActivityDrawerSlot::LeftBottom => "left_bottom",
        ActivityDrawerSlot::RightTop => "right_top",
        ActivityDrawerSlot::RightBottom => "right_bottom",
        ActivityDrawerSlot::BottomLeft => "bottom_left",
        ActivityDrawerSlot::BottomRight => "bottom_right",
    }
}

fn host_tab_icon(page: &HostPageTabModel) -> &'static str {
    if page.title == "Workbench" {
        "scene"
    } else if page.title.contains("Prefab") {
        "prefab"
    } else {
        "asset-browser"
    }
}

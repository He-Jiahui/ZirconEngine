use std::collections::BTreeMap;

use crate::ui::workbench::layout::ActivityDrawerSlot;
use crate::ui::workbench::snapshot::EditorChromeSnapshot;

use super::super::pane_tab::pane_tab_model;
use super::super::tool_window_stack_model::ToolWindowStackModel;

pub(super) fn build_tool_windows(
    chrome: &EditorChromeSnapshot,
) -> BTreeMap<ActivityDrawerSlot, ToolWindowStackModel> {
    chrome
        .workbench
        .drawers
        .iter()
        .map(|(slot, drawer)| {
            (
                *slot,
                ToolWindowStackModel {
                    slot: *slot,
                    mode: drawer.mode,
                    visible: drawer.visible,
                    active_tab: drawer.active_tab.clone(),
                    tabs: drawer
                        .tabs
                        .iter()
                        .map(|tab| {
                            pane_tab_model(
                                tab,
                                drawer.active_tab.as_ref() == Some(&tab.instance_id),
                                chrome,
                            )
                        })
                        .collect(),
                },
            )
        })
        .collect()
}

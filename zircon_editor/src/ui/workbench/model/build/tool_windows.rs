use std::collections::BTreeMap;

use crate::ui::workbench::layout::ActivityDrawerSlot;
use crate::ui::workbench::snapshot::EditorChromeSnapshot;

use super::super::pane_tab::pane_tab_model;
use super::super::tool_window_stack_model::ToolWindowStackModel;

/// 投影当前chrome中的抽屉；折叠保留标签栈，不能当成关闭视图。
pub(super) fn build_tool_windows(
    chrome: &EditorChromeSnapshot,
) -> BTreeMap<ActivityDrawerSlot, ToolWindowStackModel> {
    let mut output = BTreeMap::new();
    for (slot, drawer) in &chrome.workbench.drawers {
        let mut tabs = Vec::with_capacity(drawer.tabs.len());
        for tab in &drawer.tabs {
            tabs.push(pane_tab_model(
                tab,
                drawer.active_tab.as_ref() == Some(&tab.instance_id),
                chrome,
            ));
        }
        output.insert(
            *slot,
            ToolWindowStackModel {
                slot: *slot,
                mode: drawer.mode,
                visible: drawer.visible,
                active_tab: drawer.active_tab.clone(),
                tabs,
            },
        );
    }
    output
}

#[cfg(test)]
#[path = "tests/tool_windows_optimization_batch_20260830bz_editor_tests.rs"]
mod optimization_batch_20260830bz_editor_tests;

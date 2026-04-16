use crate::layout::{ActivityDrawerMode, ActivityDrawerSlot};
use crate::workbench::model::{DocumentTabModel, PaneTabModel, WorkbenchViewModel};

pub(super) fn active_tool_tab<'a>(
    model: &'a WorkbenchViewModel,
    slots: &[ActivityDrawerSlot],
) -> Option<&'a PaneTabModel> {
    slots
        .iter()
        .filter_map(|slot| model.tool_windows.get(slot))
        .find(|stack| {
            stack.visible && stack.mode != ActivityDrawerMode::Collapsed && !stack.tabs.is_empty()
        })
        .or_else(|| {
            slots
                .iter()
                .filter_map(|slot| model.tool_windows.get(slot))
                .find(|stack| stack.visible && !stack.tabs.is_empty())
        })
        .and_then(|stack| {
            stack
                .tabs
                .iter()
                .find(|tab| tab.active)
                .or_else(|| stack.tabs.first())
        })
}

pub(super) fn active_document_tab(model: &WorkbenchViewModel) -> Option<&DocumentTabModel> {
    model
        .document_tabs
        .iter()
        .find(|tab| tab.active)
        .or_else(|| model.document_tabs.first())
}

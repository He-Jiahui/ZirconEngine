use crate::ui::workbench::layout::{ActivityDrawerMode, ActivityDrawerSlot};
use crate::ui::workbench::model::{
    DocumentTabModel, PaneTabModel, ToolWindowStackModel, WorkbenchViewModel,
};

pub(super) fn active_tool_tab<'a>(
    model: &'a WorkbenchViewModel,
    slots: &[ActivityDrawerSlot],
) -> Option<&'a PaneTabModel> {
    preferred_tool_stack(slots.iter().filter_map(|slot| model.tool_windows.get(slot))).and_then(
        |stack| {
            stack
                .tabs
                .iter()
                .find(|tab| tab.active)
                .or_else(|| stack.tabs.first())
        },
    )
}

fn preferred_tool_stack<'a>(
    stacks: impl IntoIterator<Item = &'a ToolWindowStackModel>,
) -> Option<&'a ToolWindowStackModel> {
    let mut fallback = None;
    for stack in stacks {
        if !stack.visible || stack.tabs.is_empty() {
            continue;
        }
        if stack.mode != ActivityDrawerMode::Collapsed {
            return Some(stack);
        }
        fallback.get_or_insert(stack);
    }
    fallback
}

pub(super) fn active_document_tab(model: &WorkbenchViewModel) -> Option<&DocumentTabModel> {
    model
        .document_tabs
        .iter()
        .find(|tab| tab.active)
        .or_else(|| model.document_tabs.first())
}

#[cfg(test)]
#[path = "active_tab/single_pass_tests.rs"]
mod single_pass_tests;

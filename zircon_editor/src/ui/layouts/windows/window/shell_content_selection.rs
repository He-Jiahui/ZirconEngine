use crate::ui::workbench::layout::{ActivityDrawerMode, ActivityDrawerSlot};
use crate::ui::workbench::model::{
    DocumentTabModel, PaneTabModel, ToolWindowStackModel, WorkbenchViewModel,
};

pub(crate) struct SidePaneSelection<'a> {
    pub(crate) stack: &'a ToolWindowStackModel,
    pub(crate) tab: &'a PaneTabModel,
}

pub(crate) fn side_pane_selection<'a>(
    model: &'a WorkbenchViewModel,
    slots: &[ActivityDrawerSlot],
) -> Option<SidePaneSelection<'a>> {
    let mut first_nonempty = None;
    let mut first_active = None;
    for stack in slots.iter().filter_map(|slot| model.tool_windows.get(slot)) {
        if stack.tabs.is_empty() {
            continue;
        }
        if first_nonempty.is_none() {
            first_nonempty = Some(stack);
        }
        if stack.active_tab.is_some() {
            if stack.mode != ActivityDrawerMode::Collapsed {
                return selection_from_stack(stack);
            }
            if first_active.is_none() {
                first_active = Some(stack);
            }
        }
    }
    let stack = first_active.or(first_nonempty)?;
    selection_from_stack(stack)
}

fn selection_from_stack(stack: &ToolWindowStackModel) -> Option<SidePaneSelection<'_>> {
    let tab = stack
        .tabs
        .iter()
        .find(|tab| tab.active)
        .or_else(|| stack.tabs.first())?;
    Some(SidePaneSelection { stack, tab })
}

pub(crate) fn document_pane_selection(model: &WorkbenchViewModel) -> Option<&DocumentTabModel> {
    model
        .document_tabs
        .iter()
        .find(|tab| tab.active)
        .or_else(|| model.document_tabs.first())
}

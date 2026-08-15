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
    let stack = slots
        .iter()
        .filter_map(|slot| model.tool_windows.get(slot))
        .find(|stack| {
            stack.mode != ActivityDrawerMode::Collapsed
                && stack.active_tab.is_some()
                && !stack.tabs.is_empty()
        })
        .or_else(|| {
            slots
                .iter()
                .filter_map(|slot| model.tool_windows.get(slot))
                .find(|stack| stack.active_tab.is_some() && !stack.tabs.is_empty())
        })
        .or_else(|| {
            slots
                .iter()
                .filter_map(|slot| model.tool_windows.get(slot))
                .find(|stack| !stack.tabs.is_empty())
        })?;
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

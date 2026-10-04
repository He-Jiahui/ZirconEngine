use crate::ui::workbench::layout::{ActivityDrawerMode, ActivityDrawerSlot};
use crate::ui::workbench::model::{
    DocumentTabModel, PaneTabModel, ToolWindowStackModel, WorkbenchViewModel,
};

/// 选取本壳区域的代表工具tab供约束合并；slot顺序决定同优先级回退，不改变焦点。
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

/// 展开stack优先于折叠回退，且忽略不可见/空stack；选择顺序沿调用方传入的区域slot。
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

/// 文档约束与视口chrome使用同一活动内容；无active标记时沿模型顺序回退。
pub(super) fn active_document_tab(model: &WorkbenchViewModel) -> Option<&DocumentTabModel> {
    model
        .document_tabs
        .iter()
        .find(|tab| tab.active)
        .or_else(|| model.document_tabs.first())
}

#[cfg(test)]
#[path = "active_tab/tests/single_pass_tests.rs"]
mod single_pass_tests;

use crate::ui::workbench::model::WorkbenchViewModel;
use crate::ui::workbench::snapshot::ViewContentKind;

pub(super) fn should_collect_payload_for_kind(
    model: &WorkbenchViewModel,
    kind: ViewContentKind,
) -> bool {
    model
        .document_tabs
        .iter()
        .any(|tab| tab.active && tab.content_kind == kind)
        || model.tool_windows.values().any(|stack| {
            stack.visible
                && stack.tabs.iter().any(|tab| {
                    (tab.active || stack.active_tab.as_ref() == Some(&tab.instance_id))
                        && tab.content_kind == kind
                })
        })
        || model.floating_windows.iter().any(|window| {
            window
                .tabs
                .iter()
                .any(|tab| tab.active && tab.content_kind == kind)
        })
}

use crate::ui::workbench::model::WorkbenchViewModel;
use crate::ui::workbench::snapshot::ViewContentKind;

const DIAGNOSTIC_CONTENT_KINDS: [ViewContentKind; 2] = [
    ViewContentKind::RuntimeDiagnostics,
    ViewContentKind::PerformanceTimeline,
];

pub(super) fn should_collect_runtime_diagnostics(model: &WorkbenchViewModel) -> bool {
    model
        .document_tabs
        .iter()
        .any(active_diagnostic_document_tab)
        || model
            .tool_windows
            .values()
            .any(visible_diagnostic_tool_stack)
        || model
            .floating_windows
            .iter()
            .any(|window| window.tabs.iter().any(active_diagnostic_document_tab))
}

fn active_diagnostic_document_tab(tab: &crate::ui::workbench::model::DocumentTabModel) -> bool {
    tab.active && is_diagnostic_content_kind(tab.content_kind)
}

fn visible_diagnostic_tool_stack(
    stack: &crate::ui::workbench::model::ToolWindowStackModel,
) -> bool {
    stack.visible
        && stack.tabs.iter().any(|tab| {
            (tab.active || stack.active_tab.as_ref() == Some(&tab.instance_id))
                && is_diagnostic_content_kind(tab.content_kind)
        })
}

fn is_diagnostic_content_kind(kind: ViewContentKind) -> bool {
    DIAGNOSTIC_CONTENT_KINDS.contains(&kind)
}

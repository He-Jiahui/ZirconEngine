use super::super::super::data::TemplatePaneNodeData;
use super::super::template_node_images::is_icon_only_node;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn should_skip_template_text(
    node: &TemplatePaneNodeData,
    label: &str,
    property_row_text_painted: bool,
    table_row_text_painted: bool,
) -> bool {
    should_skip_template_text_before_label(node, property_row_text_painted, table_row_text_painted)
        || (label.is_empty() && !fallback_text_role_allows_empty_label(node))
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn should_skip_template_text_before_label(
    node: &TemplatePaneNodeData,
    property_row_text_painted: bool,
    table_row_text_painted: bool,
) -> bool {
    property_row_text_painted
        || table_row_text_painted
        || is_native_painter_owner(node)
        || (is_icon_only_node(node) && !fallback_text_role_allows_empty_label(node))
}

fn is_native_painter_owner(node: &TemplatePaneNodeData) -> bool {
    matches!(
        node.role.as_str(),
        "AgentChat"
            | "AgentPlan"
            | "AgentApproval"
            | "AIUsage"
            | "ChatComposer"
            | "DataGrid"
            | "TreeView"
            | "CommandPalette"
            | "ConfirmDialog"
            | "Dialog"
            | "DragOverlay"
            | "NotificationCenter"
            | "ToolCalls"
            | "WorkbenchToast"
    ) || matches!(
        node.component_role.as_str(),
        "mui-x-agent-chat"
            | "mui-x-agent-plan"
            | "mui-x-agent-approval"
            | "mui-x-ai-usage"
            | "mui-x-chat-composer"
            | "mui-x-data-grid"
            | "mui-x-tree-view"
            | "mui-x-command-palette"
            | "mui-x-confirm-dialog"
            | "mui-x-dialog"
            | "mui-x-drag-overlay"
            | "mui-x-notification-center"
            | "mui-x-tool-calls"
            | "mui-x-workbench-toast"
    )
}

fn fallback_text_role_allows_empty_label(node: &TemplatePaneNodeData) -> bool {
    matches!(node.role.as_str(), "Label" | "Button")
}

#[cfg(test)]
#[path = "tests/eligibility.rs"]
mod tests;

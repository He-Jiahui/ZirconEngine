//! 告知 fallback 哪些语义表面属于浮层，避免按普通内容的布局边界处理；打开状态仍由节点/浮层生命周期负责。

use super::super::super::data::TemplatePaneNodeData;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn is_mui_overlay_surface_node(
    node: &TemplatePaneNodeData,
) -> bool {
    matches!(
        node.component_role.as_str(),
        "paper"
            | "dialog"
            | "alert-dialog"
            | "popover"
            | "menu"
            | "tooltip"
            | "snackbar"
            | "drawer"
    )
}

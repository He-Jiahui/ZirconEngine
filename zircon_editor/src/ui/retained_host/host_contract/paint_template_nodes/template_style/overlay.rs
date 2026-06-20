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

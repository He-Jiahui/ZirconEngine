use super::super::super::data::TemplatePaneNodeData;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn is_drag_overlay(
    node: &TemplatePaneNodeData,
) -> bool {
    node.role.as_str() == "DragOverlay" || node.component_role.as_str() == "drag-overlay"
}

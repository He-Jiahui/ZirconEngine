use super::super::super::data::TemplatePaneNodeData;

const TEMPLATE_NODE_ORDER_STRIDE: i32 = 4;
const TEMPLATE_NODE_Z_LAYER_STRIDE: i32 = 100_000;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn template_node_paint_order(
    node: &TemplatePaneNodeData,
    row_order: i32,
) -> i32 {
    node.z_index
        .saturating_mul(TEMPLATE_NODE_Z_LAYER_STRIDE)
        .saturating_add(row_order.saturating_mul(TEMPLATE_NODE_ORDER_STRIDE))
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn template_node_transition_opacity(
    node: &TemplatePaneNodeData,
) -> f32 {
    match node.transition_kind.as_str() {
        "fade" | "grow" | "zoom" => node.transition_progress.clamp(0.0, 1.0),
        _ => 1.0,
    }
}

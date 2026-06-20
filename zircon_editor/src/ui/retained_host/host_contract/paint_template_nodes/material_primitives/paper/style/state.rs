use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;
use crate::ui::retained_host::host_contract::paint_template_nodes::material_primitives::component_variant_contains;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn paper_is_outlined(
    node: &TemplatePaneNodeData,
) -> bool {
    component_variant_contains(node, "outlined")
        || node.surface_variant.as_str() == "paper-outlined"
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn paper_elevation(
    node: &TemplatePaneNodeData,
) -> f32 {
    if node.elevation.is_finite() {
        node.elevation.max(0.0)
    } else {
        0.0
    }
}

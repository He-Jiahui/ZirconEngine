use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;

use super::super::super::super::component_variant_contains;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes::material_primitives::chip::style) fn chip_color_token(
    node: &TemplatePaneNodeData,
) -> &str {
    if component_variant_contains(node, "primary")
        || component_variant_contains(node, "colorPrimary")
    {
        "primary"
    } else if component_variant_contains(node, "secondary")
        || component_variant_contains(node, "colorSecondary")
    {
        "secondary"
    } else if component_variant_contains(node, "error")
        || component_variant_contains(node, "colorError")
    {
        "error"
    } else if component_variant_contains(node, "info")
        || component_variant_contains(node, "colorInfo")
    {
        "info"
    } else if component_variant_contains(node, "success")
        || component_variant_contains(node, "colorSuccess")
    {
        "success"
    } else if component_variant_contains(node, "warning")
        || component_variant_contains(node, "colorWarning")
    {
        "warning"
    } else {
        "default"
    }
}

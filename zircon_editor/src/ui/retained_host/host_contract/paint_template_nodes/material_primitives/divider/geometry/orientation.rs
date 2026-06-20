use super::super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::component_variant_contains;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn divider_is_vertical(
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
) -> bool {
    component_variant_contains(node, "vertical")
        || component_variant_contains(node, "wrapperVertical")
        || (!component_variant_contains(node, "horizontal") && rect.height > rect.width * 1.4)
}

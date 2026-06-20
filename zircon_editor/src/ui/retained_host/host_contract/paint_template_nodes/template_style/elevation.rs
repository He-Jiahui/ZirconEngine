use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::state::is_button_disabled;

const MATERIAL_ELEVATION_SHADOW_OFFSET: f32 = 2.0;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn draws_elevation_shadow(
    node: &TemplatePaneNodeData,
) -> bool {
    node.elevation > 0.0 && !is_button_disabled(node)
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn elevation_shadow_rect(
    rect: &FrameRect,
    elevation: f32,
) -> FrameRect {
    let offset = elevation.max(1.0) * MATERIAL_ELEVATION_SHADOW_OFFSET;
    FrameRect {
        x: rect.x + offset,
        y: rect.y + offset,
        width: rect.width,
        height: rect.height,
    }
}

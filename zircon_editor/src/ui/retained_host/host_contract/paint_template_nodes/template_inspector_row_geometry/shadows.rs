use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::metrics::{
    INSPECTOR_CHECK_SIZE, INSPECTOR_NESTED_LABEL_WIDTH,
    INSPECTOR_SHADOW_CHECK_DEFAULT_CONTENT_OFFSET_X,
};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn shadow_check_rect(
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
) -> FrameRect {
    FrameRect {
        x: rect.x + INSPECTOR_NESTED_LABEL_WIDTH + shadow_check_content_offset_x(node),
        y: rect.y + (rect.height - INSPECTOR_CHECK_SIZE).max(0.0) * 0.5,
        width: INSPECTOR_CHECK_SIZE,
        height: INSPECTOR_CHECK_SIZE,
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn shadow_check_content_offset_x(
    node: &TemplatePaneNodeData,
) -> f32 {
    let declared_offset = node.layout_content_offset_x;
    if declared_offset.is_finite() && declared_offset > 0.0 {
        declared_offset
    } else {
        INSPECTOR_SHADOW_CHECK_DEFAULT_CONTENT_OFFSET_X
    }
}

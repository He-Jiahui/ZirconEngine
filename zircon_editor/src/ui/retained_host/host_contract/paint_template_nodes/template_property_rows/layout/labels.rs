use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::identity::is_component_property_row;
use super::metrics::{
    COMPONENT_PROPERTY_LABEL_WIDTH, PROPERTY_LABEL_MAX_WIDTH_RATIO, PROPERTY_LABEL_MIN_WIDTH,
    PROPERTY_LABEL_WIDTH, PROPERTY_TEXT_INSET_X, PROPERTY_TEXT_INSET_Y,
};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn property_label_width(
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
) -> f32 {
    let preferred = if is_component_property_row(node) {
        COMPONENT_PROPERTY_LABEL_WIDTH
    } else {
        PROPERTY_LABEL_WIDTH
    };
    preferred
        .max(PROPERTY_LABEL_MIN_WIDTH)
        .min(rect.width * PROPERTY_LABEL_MAX_WIDTH_RATIO)
        .max(1.0)
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn label_text_rect(
    rect: &FrameRect,
    label_width: f32,
) -> FrameRect {
    FrameRect {
        x: rect.x + PROPERTY_TEXT_INSET_X,
        y: rect.y + PROPERTY_TEXT_INSET_Y,
        width: (label_width - PROPERTY_TEXT_INSET_X * 1.5).max(1.0),
        height: (rect.height - PROPERTY_TEXT_INSET_Y * 2.0).max(1.0),
    }
}

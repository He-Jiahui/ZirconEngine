use super::super::super::super::data::FrameRect;
use super::metrics::{PROPERTY_FIELD_INSET_Y, PROPERTY_TEXT_INSET_X, PROPERTY_TEXT_INSET_Y};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn property_value_area_rect(
    rect: &FrameRect,
    label_width: f32,
) -> FrameRect {
    FrameRect {
        x: rect.x + label_width,
        y: rect.y,
        width: (rect.width - label_width - PROPERTY_TEXT_INSET_X).max(1.0),
        height: rect.height,
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn scalar_field_rect(
    rect: &FrameRect,
) -> FrameRect {
    FrameRect {
        x: rect.x,
        y: rect.y + PROPERTY_FIELD_INSET_Y,
        width: rect.width,
        height: (rect.height - PROPERTY_FIELD_INSET_Y * 2.0).max(1.0),
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn value_text_rect(
    rect: &FrameRect,
) -> FrameRect {
    FrameRect {
        x: rect.x + PROPERTY_TEXT_INSET_X,
        y: rect.y + PROPERTY_TEXT_INSET_Y,
        width: (rect.width - PROPERTY_TEXT_INSET_X * 2.0).max(1.0),
        height: (rect.height - PROPERTY_TEXT_INSET_Y * 2.0).max(1.0),
    }
}

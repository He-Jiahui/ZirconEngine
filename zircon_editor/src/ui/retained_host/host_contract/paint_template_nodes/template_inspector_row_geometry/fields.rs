use super::super::super::data::FrameRect;
use super::metrics::{
    INSPECTOR_CHEVRON_RIGHT_PAD, INSPECTOR_CHEVRON_SIZE, INSPECTOR_COUNT_WIDTH,
    INSPECTOR_FIELD_INSET_Y, INSPECTOR_ICON_SIZE, INSPECTOR_NESTED_LABEL_WIDTH,
    INSPECTOR_NESTED_SELECT_OFFSET_X,
};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn nested_select_field_rect(
    rect: &FrameRect,
) -> FrameRect {
    let left_offset =
        INSPECTOR_NESTED_LABEL_WIDTH + INSPECTOR_COUNT_WIDTH + INSPECTOR_NESTED_SELECT_OFFSET_X;
    field_rect(rect, left_offset, rect.width - left_offset)
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn field_rect(
    rect: &FrameRect,
    left_offset: f32,
    width: f32,
) -> FrameRect {
    FrameRect {
        x: rect.x + left_offset,
        y: rect.y + INSPECTOR_FIELD_INSET_Y,
        width: width.max(1.0),
        height: (rect.height - INSPECTOR_FIELD_INSET_Y * 2.0).max(1.0),
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn leading_affordance_rect(
    field: &FrameRect,
) -> FrameRect {
    FrameRect {
        x: field.x + 8.0,
        y: field.y + (field.height - INSPECTOR_ICON_SIZE).max(0.0) * 0.5,
        width: INSPECTOR_ICON_SIZE,
        height: INSPECTOR_ICON_SIZE,
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn chevron_rect(
    field: &FrameRect,
    size: f32,
) -> FrameRect {
    let size = if size.is_finite() && size > 0.0 {
        size
    } else {
        INSPECTOR_CHEVRON_SIZE
    };
    FrameRect {
        x: field.x + field.width - size - INSPECTOR_CHEVRON_RIGHT_PAD,
        y: field.y + (field.height - size).max(0.0) * 0.5,
        width: size,
        height: size,
    }
}

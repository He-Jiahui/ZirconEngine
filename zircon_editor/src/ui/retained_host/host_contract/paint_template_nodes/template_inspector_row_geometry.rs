use super::super::data::{FrameRect, TemplatePaneNodeData};

pub(super) const INSPECTOR_ROW_TEXT_Y: f32 = 5.0;
pub(super) const INSPECTOR_LABEL_WIDTH: f32 = 104.0;
pub(super) const INSPECTOR_COUNT_WIDTH: f32 = 24.0;
pub(super) const INSPECTOR_FIELD_TEXT_X: f32 = 8.0;
pub(super) const INSPECTOR_FIELD_RIGHT_PAD: f32 = 22.0;
pub(super) const INSPECTOR_CHEVRON_SIZE: f32 = 10.0;

const INSPECTOR_NESTED_LABEL_WIDTH: f32 = 116.0;
const INSPECTOR_NESTED_LABEL_BASE_X: f32 = 6.0;
const INSPECTOR_NESTED_LABEL_OFFSET_X: f32 = 8.0;
const INSPECTOR_NESTED_SELECT_OFFSET_X: f32 = 14.0;
const INSPECTOR_FIELD_INSET_Y: f32 = 3.0;
const INSPECTOR_ICON_SIZE: f32 = 13.0;
const INSPECTOR_CHEVRON_RIGHT_PAD: f32 = 5.0;
const INSPECTOR_CHECK_SIZE: f32 = 14.0;
const INSPECTOR_SHADOW_CHECK_DEFAULT_CONTENT_OFFSET_X: f32 = INSPECTOR_COUNT_WIDTH + 4.0;

pub(super) fn shadow_check_rect(node: &TemplatePaneNodeData, rect: &FrameRect) -> FrameRect {
    FrameRect {
        x: rect.x + INSPECTOR_NESTED_LABEL_WIDTH + shadow_check_content_offset_x(node),
        y: rect.y + (rect.height - INSPECTOR_CHECK_SIZE).max(0.0) * 0.5,
        width: INSPECTOR_CHECK_SIZE,
        height: INSPECTOR_CHECK_SIZE,
    }
}

pub(super) fn shadow_check_content_offset_x(node: &TemplatePaneNodeData) -> f32 {
    let declared_offset = node.layout_content_offset_x;
    if declared_offset.is_finite() && declared_offset > 0.0 {
        declared_offset
    } else {
        INSPECTOR_SHADOW_CHECK_DEFAULT_CONTENT_OFFSET_X
    }
}

pub(super) fn nested_label_rect(rect: &FrameRect) -> FrameRect {
    let x = rect.x + INSPECTOR_NESTED_LABEL_BASE_X + INSPECTOR_NESTED_LABEL_OFFSET_X;
    FrameRect {
        x,
        y: rect.y + INSPECTOR_ROW_TEXT_Y,
        width: (INSPECTOR_NESTED_LABEL_WIDTH - (x - rect.x) - 4.0).max(1.0),
        height: (rect.height - INSPECTOR_ROW_TEXT_Y * 2.0).max(1.0),
    }
}

pub(super) fn nested_select_field_rect(rect: &FrameRect) -> FrameRect {
    let left_offset =
        INSPECTOR_NESTED_LABEL_WIDTH + INSPECTOR_COUNT_WIDTH + INSPECTOR_NESTED_SELECT_OFFSET_X;
    field_rect(rect, left_offset, rect.width - left_offset)
}

pub(super) fn field_rect(rect: &FrameRect, left_offset: f32, width: f32) -> FrameRect {
    FrameRect {
        x: rect.x + left_offset,
        y: rect.y + INSPECTOR_FIELD_INSET_Y,
        width: width.max(1.0),
        height: (rect.height - INSPECTOR_FIELD_INSET_Y * 2.0).max(1.0),
    }
}

pub(super) fn leading_affordance_rect(field: &FrameRect) -> FrameRect {
    FrameRect {
        x: field.x + 8.0,
        y: field.y + (field.height - INSPECTOR_ICON_SIZE).max(0.0) * 0.5,
        width: INSPECTOR_ICON_SIZE,
        height: INSPECTOR_ICON_SIZE,
    }
}

pub(super) fn chevron_rect(field: &FrameRect, size: f32) -> FrameRect {
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

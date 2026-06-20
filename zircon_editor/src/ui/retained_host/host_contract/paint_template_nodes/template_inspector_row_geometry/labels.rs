use super::super::super::data::FrameRect;
use super::metrics::{
    INSPECTOR_NESTED_LABEL_BASE_X, INSPECTOR_NESTED_LABEL_OFFSET_X, INSPECTOR_NESTED_LABEL_WIDTH,
    INSPECTOR_ROW_TEXT_Y,
};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn nested_label_rect(
    rect: &FrameRect,
) -> FrameRect {
    let x = rect.x + INSPECTOR_NESTED_LABEL_BASE_X + INSPECTOR_NESTED_LABEL_OFFSET_X;
    FrameRect {
        x,
        y: rect.y + INSPECTOR_ROW_TEXT_Y,
        width: (INSPECTOR_NESTED_LABEL_WIDTH - (x - rect.x) - 4.0).max(1.0),
        height: (rect.height - INSPECTOR_ROW_TEXT_Y * 2.0).max(1.0),
    }
}

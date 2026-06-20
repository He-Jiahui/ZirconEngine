use super::super::super::data::FrameRect;

const POPUP_ROW_ADORNMENT_RIGHT: f32 = 12.0;
const POPUP_ROW_ADORNMENT_SIZE: f32 = 14.0;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const POPUP_ROW_ADORNMENT_RESERVED_WIDTH: f32 =
    30.0;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn popup_row_adornment_rect(
    row_rect: &FrameRect,
) -> FrameRect {
    FrameRect {
        x: row_rect.x + row_rect.width - POPUP_ROW_ADORNMENT_RIGHT - POPUP_ROW_ADORNMENT_SIZE,
        y: row_rect.y + (row_rect.height - POPUP_ROW_ADORNMENT_SIZE).max(0.0) * 0.5,
        width: POPUP_ROW_ADORNMENT_SIZE,
        height: POPUP_ROW_ADORNMENT_SIZE,
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn local_rect(
    origin: &FrameRect,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) -> FrameRect {
    FrameRect {
        x: origin.x + x,
        y: origin.y + y,
        width,
        height,
    }
}

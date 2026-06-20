use super::super::super::data::FrameRect;

const LIST_ROW_RIGHT_INSET: f32 = 12.0;
const LIST_ROW_ADORNMENT_SIZE: f32 = 13.0;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn list_row_adornment_rect(
    rect: &FrameRect,
) -> FrameRect {
    FrameRect {
        x: rect.x + rect.width - LIST_ROW_RIGHT_INSET - LIST_ROW_ADORNMENT_SIZE,
        y: rect.y + (rect.height - LIST_ROW_ADORNMENT_SIZE).max(0.0) * 0.5,
        width: LIST_ROW_ADORNMENT_SIZE,
        height: LIST_ROW_ADORNMENT_SIZE,
    }
}

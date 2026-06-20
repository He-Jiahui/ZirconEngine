use super::super::super::data::FrameRect;
use super::metrics::status_line_height;

const STATUS_CHIP_TEXT_LEFT: f32 = 12.0;
const STATUS_CHIP_RIGHT_RESERVE: f32 = 24.0;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn status_chip_text_rect(
    rect: &FrameRect,
) -> FrameRect {
    let line_height = status_line_height();
    FrameRect {
        x: rect.x + STATUS_CHIP_TEXT_LEFT,
        y: rect.y + (rect.height - line_height).max(0.0) * 0.5,
        width: (rect.width - STATUS_CHIP_TEXT_LEFT - STATUS_CHIP_RIGHT_RESERVE).max(1.0),
        height: line_height,
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn status_chip_chevron_rect(
    rect: &FrameRect,
) -> FrameRect {
    FrameRect {
        x: rect.x + rect.width - 18.0,
        y: rect.y + (rect.height - 12.0).max(0.0) * 0.5,
        width: 12.0,
        height: 12.0,
    }
}

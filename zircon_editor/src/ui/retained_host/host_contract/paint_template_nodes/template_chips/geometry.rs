use super::super::super::data::FrameRect;
use super::style::CHIP_LINE_HEIGHT;

const CHIP_TEXT_LEFT: f32 = 10.0;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const CHIP_TEXT_RIGHT: f32 =
    8.0;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn chip_label_rect(
    rect: &FrameRect,
    right_reserve: f32,
) -> FrameRect {
    FrameRect {
        x: rect.x + CHIP_TEXT_LEFT,
        y: rect.y + (rect.height - CHIP_LINE_HEIGHT).max(0.0) * 0.5,
        width: (rect.width - CHIP_TEXT_LEFT - right_reserve).max(1.0),
        height: CHIP_LINE_HEIGHT,
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn pixel_aligned_rect(
    rect: &FrameRect,
) -> FrameRect {
    FrameRect {
        x: rect.x.round(),
        y: rect.y.round(),
        width: rect.width.round().max(1.0),
        height: rect.height.round().max(1.0),
    }
}

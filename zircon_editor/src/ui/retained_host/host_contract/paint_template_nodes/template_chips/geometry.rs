use super::super::super::data::FrameRect;
use super::metrics::{chip_line_height, chip_text_left};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn chip_label_rect(
    rect: &FrameRect,
    right_reserve: f32,
) -> FrameRect {
    let line_height = chip_line_height();
    let text_left = chip_text_left();
    FrameRect {
        x: rect.x + text_left,
        y: rect.y + (rect.height - line_height).max(0.0) * 0.5,
        width: (rect.width - text_left - right_reserve).max(1.0),
        height: line_height,
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

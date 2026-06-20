use super::super::super::data::FrameRect;
use super::super::template_section_title_glyphs::{SECTION_ICON_GAP, SECTION_ICON_SIZE};
use super::style::SECTION_LINE_HEIGHT;

const SECTION_TEXT_LEFT: f32 = 8.0;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn section_icon_rect(
    rect: &FrameRect,
) -> FrameRect {
    FrameRect {
        x: rect.x + SECTION_TEXT_LEFT,
        y: rect.y + (rect.height - SECTION_ICON_SIZE).max(0.0) * 0.5,
        width: SECTION_ICON_SIZE,
        height: SECTION_ICON_SIZE,
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn section_label_rect(
    rect: &FrameRect,
    has_icon: bool,
) -> FrameRect {
    let x = if has_icon {
        rect.x + SECTION_TEXT_LEFT + SECTION_ICON_SIZE + SECTION_ICON_GAP
    } else {
        rect.x + SECTION_TEXT_LEFT
    };
    FrameRect {
        x,
        y: rect.y + (rect.height - SECTION_LINE_HEIGHT).max(0.0) * 0.5,
        width: (rect.x + rect.width - x - SECTION_TEXT_LEFT).max(1.0),
        height: SECTION_LINE_HEIGHT,
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

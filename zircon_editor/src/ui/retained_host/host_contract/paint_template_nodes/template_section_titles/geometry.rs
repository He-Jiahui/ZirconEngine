use super::super::super::data::FrameRect;
use super::super::template_section_title_glyphs::section_title_glyph_metrics;
use super::style::section_title_metrics;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn section_icon_rect(
    rect: &FrameRect,
) -> FrameRect {
    let glyph_metrics = section_title_glyph_metrics();
    let section_metrics = section_title_metrics();
    FrameRect {
        x: rect.x + section_metrics.text_left,
        y: rect.y + (rect.height - glyph_metrics.icon_size).max(0.0) * 0.5,
        width: glyph_metrics.icon_size,
        height: glyph_metrics.icon_size,
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn section_label_rect(
    rect: &FrameRect,
    has_icon: bool,
) -> FrameRect {
    let glyph_metrics = section_title_glyph_metrics();
    let section_metrics = section_title_metrics();
    let x = if has_icon {
        rect.x + section_metrics.text_left + glyph_metrics.icon_size + glyph_metrics.icon_gap
    } else {
        rect.x + section_metrics.text_left
    };
    FrameRect {
        x,
        y: rect.y + (rect.height - section_metrics.line_height).max(0.0) * 0.5,
        width: (rect.x + rect.width - x - section_metrics.text_left).max(1.0),
        height: section_metrics.line_height,
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

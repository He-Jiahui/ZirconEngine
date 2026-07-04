use super::super::super::data::FrameRect;
use super::super::template_status_glyphs::centered_rect;
use super::status_icon_glyph_size;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn status_icon_button_glyph_rect(
    rect: &FrameRect,
) -> FrameRect {
    centered_rect(rect, status_icon_glyph_size())
}

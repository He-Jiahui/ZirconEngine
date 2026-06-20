use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::render_commands::HostPaintCommand;
use super::super::super::template_button_glyphs::{
    button_glyph_for_key, push_button_glyph, ButtonGlyph, BUTTON_ICON_SIZE,
};
use super::super::identity::button_key;
use super::metrics::{BUTTON_CHEVRON_RESERVE, BUTTON_ICON_GAP, BUTTON_TEXT_INSET_X};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn button_glyph(
    node: &TemplatePaneNodeData,
) -> ButtonGlyph {
    button_glyph_for_key(&button_key(node))
}

pub(super) fn button_glyph_width(glyph: ButtonGlyph) -> f32 {
    if has_leading_glyph(glyph) {
        BUTTON_ICON_SIZE + BUTTON_ICON_GAP
    } else {
        0.0
    }
}

pub(super) fn chevron_width(glyph: ButtonGlyph) -> f32 {
    if has_trailing_chevron(glyph) {
        BUTTON_CHEVRON_RESERVE
    } else {
        0.0
    }
}

pub(super) fn has_leading_glyph(glyph: ButtonGlyph) -> bool {
    matches!(glyph, ButtonGlyph::Plus | ButtonGlyph::Trash)
}

pub(super) fn has_trailing_chevron(glyph: ButtonGlyph) -> bool {
    glyph == ButtonGlyph::ChevronDown
}

pub(super) fn leading_glyph_rect(rect: &FrameRect, x: f32) -> FrameRect {
    FrameRect {
        x,
        y: rect.y + (rect.height - BUTTON_ICON_SIZE).max(0.0) * 0.5,
        width: BUTTON_ICON_SIZE,
        height: BUTTON_ICON_SIZE,
    }
}

pub(super) fn trailing_glyph_rect(rect: &FrameRect) -> FrameRect {
    FrameRect {
        x: rect.x + rect.width - BUTTON_TEXT_INSET_X - BUTTON_ICON_SIZE,
        y: rect.y + (rect.height - BUTTON_ICON_SIZE).max(0.0) * 0.5,
        width: BUTTON_ICON_SIZE,
        height: BUTTON_ICON_SIZE,
    }
}

pub(super) fn push_content_glyph(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    glyph: ButtonGlyph,
    color: [u8; 4],
    opacity: f32,
) {
    push_button_glyph(commands, rect, clip, order, glyph, color, opacity);
}

use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::render_commands::HostPaintCommand;
use super::super::super::template_button_glyphs::{
    button_glyph_for_key, button_icon_size, push_button_glyph, ButtonGlyph,
};
use super::super::super::template_icon_assets::push_icon_asset_pixels;
use super::super::identity::button_key;
use super::layout::content_centered_y;
use super::metrics::{button_chevron_reserve, button_icon_gap, trailing_glyph_inset};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn button_glyph(
    node: &TemplatePaneNodeData,
) -> ButtonGlyph {
    button_glyph_for_key(&button_key(node))
}

pub(super) fn button_glyph_width(node: &TemplatePaneNodeData, glyph: ButtonGlyph) -> f32 {
    if has_leading_asset_icon(node) || has_leading_glyph(glyph) {
        button_icon_size() + button_icon_gap(node)
    } else {
        0.0
    }
}

pub(super) fn chevron_width(glyph: ButtonGlyph) -> f32 {
    if has_trailing_chevron(glyph) {
        button_chevron_reserve()
    } else {
        0.0
    }
}

pub(super) fn has_leading_glyph(glyph: ButtonGlyph) -> bool {
    matches!(glyph, ButtonGlyph::Plus | ButtonGlyph::Trash)
}

pub(super) fn has_leading_asset_icon(node: &TemplatePaneNodeData) -> bool {
    !node.icon_name.trim().is_empty()
}

pub(super) fn has_trailing_chevron(glyph: ButtonGlyph) -> bool {
    glyph == ButtonGlyph::ChevronDown
}

pub(super) fn leading_glyph_rect(rect: &FrameRect, x: f32) -> FrameRect {
    let icon_size = button_icon_size();
    FrameRect {
        x,
        y: content_centered_y(rect, icon_size),
        width: icon_size,
        height: icon_size,
    }
}

pub(super) fn trailing_glyph_rect(rect: &FrameRect) -> FrameRect {
    let icon_size = button_icon_size();
    FrameRect {
        x: rect.x + rect.width - trailing_glyph_inset() - icon_size,
        y: content_centered_y(rect, icon_size),
        width: icon_size,
        height: icon_size,
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

pub(super) fn push_content_asset_icon(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) -> bool {
    push_icon_asset_pixels(
        commands,
        node.icon_name.as_str(),
        rect,
        clip,
        order,
        Some(color),
        opacity,
    )
}

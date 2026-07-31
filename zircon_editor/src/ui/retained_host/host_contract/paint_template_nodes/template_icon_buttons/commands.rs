use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::render_commands::HostPaintCommand;
use super::super::template_icon_button_glyphs::push_icon_button_glyph;
use super::content::icon_button_content_style;
use super::geometry::{
    frame_is_within, has_paintable_icon_button_extent, icon_button_paint_rect,
    icon_glyph_is_paintable, icon_glyph_rect,
};
use super::identity::is_workbench_icon_button;
use super::layers::glyph_order;
use super::style::{icon_button_context, icon_button_style};
use super::surface::push_icon_button_surface;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_icon_button_commands(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) -> bool {
    if !is_workbench_icon_button(node) {
        return false;
    }

    if !has_paintable_icon_button_extent(rect) {
        return true;
    }
    let rect = icon_button_paint_rect(node, rect);
    if !frame_is_within(&rect, clip) {
        return true;
    }
    let context = icon_button_context(node);
    let style = icon_button_style(node, context);
    push_icon_button_surface(commands, &rect, clip, order, style, opacity);
    let content_style = icon_button_content_style(style);
    let glyph = content_style.offset_glyph_rect(icon_glyph_rect(node, &rect, context));
    if icon_glyph_is_paintable(&glyph, &rect, context) {
        push_icon_button_glyph(
            commands,
            node,
            &glyph,
            clip,
            glyph_order(order),
            content_style.glyph,
            opacity,
        );
    }
    true
}

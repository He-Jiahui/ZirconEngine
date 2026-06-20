use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::render_commands::HostPaintCommand;
use super::super::super::style_selector::WorkbenchButtonKind;
use super::super::super::template_node_labels::template_node_label;
use super::super::style::button_style;
use super::glyph::{
    button_glyph, button_glyph_width, chevron_width, has_leading_glyph, has_trailing_chevron,
    leading_glyph_rect, push_content_glyph, trailing_glyph_rect,
};
use super::metrics::{estimated_label_width, BUTTON_TEXT_INSET_X};
use super::text::push_button_label;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_button_content(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    kind: WorkbenchButtonKind,
    opacity: f32,
) {
    let label = template_node_label(node, None);
    let style = button_style(node, kind);
    let glyph = button_glyph(node);
    let glyph_width = button_glyph_width(glyph);
    let chevron_width = chevron_width(glyph);
    let content_width = (estimated_label_width(&label) + glyph_width + chevron_width)
        .min((rect.width - BUTTON_TEXT_INSET_X * 2.0).max(1.0));
    let mut x = rect.x + (rect.width - content_width).max(0.0) * 0.5;

    if has_leading_glyph(glyph) {
        let glyph_rect = leading_glyph_rect(rect, x);
        push_content_glyph(
            commands,
            &glyph_rect,
            clip,
            order,
            glyph,
            style.glyph,
            opacity,
        );
        x += glyph_width;
    }

    if !label.trim().is_empty() {
        push_button_label(
            commands,
            node,
            rect,
            clip,
            order + 1,
            x,
            (content_width - glyph_width - chevron_width).max(1.0),
            label,
            style.text,
            opacity,
        );
    }

    if has_trailing_chevron(glyph) {
        let glyph_rect = trailing_glyph_rect(rect);
        push_content_glyph(
            commands,
            &glyph_rect,
            clip,
            order,
            glyph,
            style.glyph,
            opacity,
        );
    }
}

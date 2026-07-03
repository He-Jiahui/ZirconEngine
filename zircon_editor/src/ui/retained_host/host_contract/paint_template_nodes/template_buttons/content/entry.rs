use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::render_commands::HostPaintCommand;
use super::super::super::style_selector::WorkbenchButtonKind;
use super::super::super::template_node_labels::template_node_label;
use super::super::style::button_style;
use super::glyph::{
    button_glyph, button_glyph_width, chevron_width, has_leading_asset_icon, has_leading_glyph,
    has_trailing_chevron, leading_glyph_rect, push_content_asset_icon, push_content_glyph,
    trailing_glyph_rect,
};
use super::metrics::{
    button_label_font_size, button_label_paint_style, content_offset_y, max_label_slot_width,
    measured_label_width,
};
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
    let content_y_offset = content_offset_y(style.interaction);
    let glyph = button_glyph(node);
    let glyph_width = button_glyph_width(node, glyph);
    let chevron_width = chevron_width(glyph);
    let font_size = button_label_font_size(node, rect);
    let text_style = button_label_paint_style(node);
    let content_width =
        (measured_label_width(&label, font_size, text_style) + glyph_width + chevron_width)
            .min(max_label_slot_width(node, rect));
    let mut x = rect.x + (rect.width - content_width).max(0.0) * 0.5;

    if has_leading_asset_icon(node) {
        let glyph_rect = offset_rect_y(leading_glyph_rect(rect, x), content_y_offset);
        let rendered_asset = push_content_asset_icon(
            commands,
            node,
            &glyph_rect,
            clip,
            order,
            style.glyph,
            opacity,
        );
        if !rendered_asset && has_leading_glyph(glyph) {
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
        x += glyph_width;
    } else if has_leading_glyph(glyph) {
        let glyph_rect = offset_rect_y(leading_glyph_rect(rect, x), content_y_offset);
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
            rect,
            clip,
            order + 1,
            x,
            content_y_offset,
            (content_width - glyph_width - chevron_width).max(1.0),
            font_size,
            text_style,
            label,
            style.text,
            opacity,
        );
    }

    if has_trailing_chevron(glyph) {
        let glyph_rect = offset_rect_y(trailing_glyph_rect(rect), content_y_offset);
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

fn offset_rect_y(mut rect: FrameRect, offset: f32) -> FrameRect {
    rect.y += offset;
    rect
}

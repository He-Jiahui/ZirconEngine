use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::render_commands::HostPaintCommand;
use super::super::super::style_selector::WorkbenchButtonKind;
use super::super::super::template_node_labels::template_node_label;
use super::super::geometry::frame_is_within;
use super::super::layers::label_order;
use super::glyph::{
    button_glyph, button_glyph_width, chevron_width, has_leading_asset_icon, has_leading_glyph,
    has_trailing_chevron, leading_glyph_rect, push_content_asset_icon, push_content_glyph,
    trailing_glyph_rect,
};
use super::layout::button_content_layout;
use super::metrics::{button_label_font_size, button_label_paint_style, measured_label_ink_width};
use super::style::button_content_style;
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
    let content_style = button_content_style(node, kind);
    let content_y_offset = content_style.y_offset;
    let glyph = button_glyph(node);
    let glyph_width = button_glyph_width(node, glyph);
    let chevron_width = chevron_width(glyph);
    let font_size = button_label_font_size(node, rect);
    let text_style = button_label_paint_style(node);
    let label_ink_width = measured_label_ink_width(&label, font_size, text_style);
    let layout = button_content_layout(node, rect, glyph_width, chevron_width, label_ink_width);
    let mut x = layout.start_x;

    if has_leading_asset_icon(node) {
        let glyph_rect = offset_rect_y(leading_glyph_rect(rect, x), content_y_offset);
        if frame_is_within(&glyph_rect, rect) {
            let rendered_asset = push_content_asset_icon(
                commands,
                node,
                &glyph_rect,
                clip,
                order,
                content_style.glyph,
                opacity,
            );
            if !rendered_asset && has_leading_glyph(glyph) {
                push_content_glyph(
                    commands,
                    &glyph_rect,
                    clip,
                    order,
                    glyph,
                    content_style.glyph,
                    opacity,
                );
            }
        }
        x += glyph_width;
    } else if has_leading_glyph(glyph) {
        let glyph_rect = offset_rect_y(leading_glyph_rect(rect, x), content_y_offset);
        if frame_is_within(&glyph_rect, rect) {
            push_content_glyph(
                commands,
                &glyph_rect,
                clip,
                order,
                glyph,
                content_style.glyph,
                opacity,
            );
        }
        x += glyph_width;
    }

    if !label.trim().is_empty() {
        push_button_label(
            commands,
            rect,
            clip,
            label_order(order),
            x,
            content_y_offset,
            layout.text_slot_width,
            font_size,
            text_style,
            label,
            content_style.text,
            opacity,
        );
    }

    if has_trailing_chevron(glyph) {
        let glyph_rect = offset_rect_y(trailing_glyph_rect(rect), content_y_offset);
        if frame_is_within(&glyph_rect, rect) {
            push_content_glyph(
                commands,
                &glyph_rect,
                clip,
                order,
                glyph,
                content_style.glyph,
                opacity,
            );
        }
    }
}

fn offset_rect_y(mut rect: FrameRect, offset: f32) -> FrameRect {
    rect.y += offset;
    rect
}

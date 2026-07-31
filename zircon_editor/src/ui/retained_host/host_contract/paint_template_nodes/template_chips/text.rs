use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::render_commands::HostPaintCommand;
use super::super::template_chip_glyphs::{
    chip_can_paint_chevron, chip_glyph_chevron_reserve, chip_has_chevron,
};
use super::super::template_node_labels::template_node_label;
use super::geometry::chip_label_rect;
use super::metrics::{chip_font_size, chip_line_height, chip_text_right};
use super::style::chip_text_color;
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_chip_label(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let label = template_node_label(node, None);
    if label.trim().is_empty() {
        return;
    }
    let right_reserve = if chip_has_chevron(node) && chip_can_paint_chevron(rect) {
        chip_glyph_chevron_reserve()
    } else {
        chip_text_right()
    };
    let font_size = chip_font_size();
    let line_height = chip_line_height();
    if rect.height < line_height {
        return;
    }
    let label_rect = chip_label_rect(rect, right_reserve);
    if label_rect.width <= 0.0 {
        return;
    }
    commands.push(HostPaintCommand::text(
        label_rect,
        Some(clip.clone()),
        order,
        label,
        chip_text_color(node),
        font_size,
        line_height,
        UiTextRunPaintStyle::default(),
        opacity,
    ));
}

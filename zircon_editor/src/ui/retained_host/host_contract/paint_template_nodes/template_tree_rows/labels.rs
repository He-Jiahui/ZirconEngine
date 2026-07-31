use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::render_commands::HostPaintCommand;
use super::super::template_node_labels::template_node_label;
use super::super::template_tree_row_geometry::{tree_font_size, tree_label_rect, tree_line_height};
use super::geometry::tree_row_contains;
use super::style::tree_text_color;
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_tree_label(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    icon: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let label = template_node_label(node, None);
    if label.trim().is_empty() {
        return;
    }

    let text_rect = tree_label_rect(rect, icon);
    if !tree_row_contains(rect, &text_rect) {
        return;
    }
    commands.push(HostPaintCommand::text(
        text_rect,
        Some(clip.clone()),
        order,
        label,
        tree_text_color(node),
        tree_font_size(),
        tree_line_height(),
        UiTextRunPaintStyle::default(),
        opacity,
    ));
}

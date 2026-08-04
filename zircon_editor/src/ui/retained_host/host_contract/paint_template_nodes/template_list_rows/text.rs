use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::paint_geometry::intersect;
use super::super::render_commands::HostPaintCommand;
use super::super::template_node_labels::template_node_label;
use super::super::template_row_metrics::workbench_row_metrics;
use super::style::list_row_text_color;
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_list_row_label(
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
    let metrics = workbench_row_metrics();
    let text_frame = FrameRect {
        x: rect.x + metrics.text_inset_x,
        y: rect.y + metrics.text_inset_y,
        width: rect.width - metrics.text_inset_x - metrics.right_reserve,
        height: rect.height - metrics.text_inset_y * 2.0,
    };
    if text_frame.width <= 0.0 || text_frame.height <= 0.0 {
        return;
    }
    let Some(text_clip) = intersect(&text_frame, clip) else {
        return;
    };
    commands.push(HostPaintCommand::text(
        text_frame,
        Some(text_clip),
        order,
        label,
        list_row_text_color(node),
        metrics.text_font_size,
        metrics.text_line_height,
        UiTextRunPaintStyle::default(),
        opacity,
    ));
}

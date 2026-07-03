use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::render_commands::HostPaintCommand;
use super::super::template_node_labels::template_node_label;
use super::super::template_selection_control_geometry::workbench_selection_control_metrics;
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_selection_label(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    let label = template_node_label(node, None);
    if label.trim().is_empty() || rect.width <= 0.5 || rect.height <= 0.5 {
        return;
    }
    let metrics = workbench_selection_control_metrics();
    commands.push(HostPaintCommand::text(
        rect,
        Some(clip.clone()),
        order,
        label,
        color,
        metrics.font_size,
        metrics.line_height,
        UiTextRunPaintStyle::default(),
        opacity,
    ));
}

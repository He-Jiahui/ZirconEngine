use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::render_commands::HostPaintCommand;
use super::super::template_axis_value_field_style::axis_field_text_color;
use super::metrics::axis_value_field_metrics;
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_axis_field_value(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    field: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let value = axis_field_value(node);
    if value.is_empty() {
        return;
    }

    let metrics = axis_value_field_metrics();
    commands.push(HostPaintCommand::text(
        FrameRect {
            x: field.x + metrics.text_inset_x,
            y: field.y + (field.height - metrics.line_height).max(0.0) * 0.5,
            width: (field.width - metrics.text_inset_x * 2.0).max(1.0),
            height: metrics.line_height,
        },
        Some(clip.clone()),
        order,
        value.to_string(),
        axis_field_text_color(node),
        metrics.font_size,
        metrics.line_height,
        UiTextRunPaintStyle::default(),
        opacity,
    ));
}

fn axis_field_value(node: &TemplatePaneNodeData) -> &str {
    let value = node.value_text.trim();
    if value.is_empty() {
        node.text.trim()
    } else {
        value
    }
}

mod geometry;
mod style;

use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::render_commands::HostPaintCommand;
use super::metrics::axis_label_metrics;

use geometry::axis_label_text_rect;
use style::axis_label_text_command_style;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_axis_text(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    axis: &str,
    opacity: f32,
) {
    let label = axis_label_text(node, axis);
    let metrics = axis_label_metrics();
    let style = axis_label_text_command_style(node, &metrics);
    commands.push(HostPaintCommand::text(
        axis_label_text_rect(rect, &metrics),
        Some(clip.clone()),
        order,
        label.to_string(),
        style.color,
        style.font_size,
        style.line_height,
        style.paint_style,
        opacity,
    ));
}

fn axis_label_text<'a>(node: &'a TemplatePaneNodeData, axis: &'a str) -> &'a str {
    let text = node.text.trim();
    if text.is_empty() {
        axis
    } else {
        text
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn text_node(control_id: &str, text: &str) -> TemplatePaneNodeData {
        TemplatePaneNodeData {
            control_id: control_id.into(),
            role: "Label".into(),
            text: text.into(),
            ..TemplatePaneNodeData::default()
        }
    }

    #[test]
    fn axis_label_text_uses_trimmed_declared_text_or_axis_fallback() {
        assert_eq!(
            axis_label_text(
                &text_node("WorkbenchTransformPositionAxisX", "  Position X  "),
                "X",
            ),
            "Position X"
        );
        assert_eq!(
            axis_label_text(&text_node("WorkbenchTransformRotationAxisY", "   "), "Y"),
            "Y"
        );
    }
}

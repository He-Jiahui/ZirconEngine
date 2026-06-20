use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::render_commands::HostPaintCommand;
use super::style::{axis_label_color, AXIS_LABEL_FONT_SIZE};
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_axis_text(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    axis: &str,
    opacity: f32,
) {
    let label = if node.text.trim().is_empty() {
        axis
    } else {
        node.text.trim()
    };
    let line_height = AXIS_LABEL_FONT_SIZE * 1.2;
    commands.push(HostPaintCommand::text(
        FrameRect {
            x: rect.x,
            y: rect.y + (rect.height - line_height).max(0.0) * 0.5,
            width: rect.width.max(1.0),
            height: line_height,
        },
        Some(clip.clone()),
        order,
        label.to_string(),
        axis_label_color(node),
        AXIS_LABEL_FONT_SIZE,
        line_height,
        UiTextRunPaintStyle::default(),
        opacity,
    ));
}

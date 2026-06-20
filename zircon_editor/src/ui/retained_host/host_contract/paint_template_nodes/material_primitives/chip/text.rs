use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::render_commands::HostPaintCommand;
use super::super::first_non_empty;
use super::geometry::chip_label_frame;
use super::style::chip_foreground_color;
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_chip_label(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let label = chip_label(node);
    if label.is_empty() {
        return;
    }
    let Some((frame, font_size, line_height)) = chip_label_frame(node, rect, &label) else {
        return;
    };
    commands.push(HostPaintCommand::text(
        frame,
        Some(clip.clone()),
        order,
        label,
        chip_foreground_color(node),
        font_size,
        line_height,
        UiTextRunPaintStyle::default(),
        opacity,
    ));
}

fn chip_label(node: &TemplatePaneNodeData) -> String {
    first_non_empty(&[node.text.as_str(), node.value_text.as_str()]).to_string()
}

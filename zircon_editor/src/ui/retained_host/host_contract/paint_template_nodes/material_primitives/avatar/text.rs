use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::render_commands::HostPaintCommand;
use super::super::first_non_empty;
use super::geometry::avatar_text_frame;
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_avatar_text(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    let label = avatar_label(node);
    if label.is_empty() {
        return;
    }
    let (frame, font_size, line_height) = avatar_text_frame(node, rect, &label);
    commands.push(HostPaintCommand::text(
        frame,
        Some(clip.clone()),
        order,
        label,
        color,
        font_size,
        line_height,
        UiTextRunPaintStyle::default(),
        opacity,
    ));
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn avatar_label(
    node: &TemplatePaneNodeData,
) -> String {
    first_non_empty(&[
        node.text.as_str(),
        node.value_text.as_str(),
        node.options_text.as_str(),
    ])
    .trim()
    .to_string()
}

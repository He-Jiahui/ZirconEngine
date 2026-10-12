use super::super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::super::super::paint_geometry::intersect;
use super::super::super::super::render_commands::HostPaintCommand;
use super::super::geometry::horizontal_label_text_frame;
use super::super::style::divider_text_color;
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_horizontal_divider_label(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    label: &str,
    label_left: f32,
    label_right: f32,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    if label.trim().is_empty() || label_right <= label_left {
        return;
    }
    let Some((frame, font_size, line_height)) =
        horizontal_label_text_frame(node, label, label_left, label_right, rect)
    else {
        return;
    };
    let Some(text_clip) = intersect(&frame, clip) else {
        return;
    };
    commands.push(HostPaintCommand::text(
        frame,
        Some(text_clip),
        order,
        label.to_string(),
        divider_text_color(node),
        font_size,
        line_height,
        UiTextRunPaintStyle::default(),
        opacity,
    ));
}

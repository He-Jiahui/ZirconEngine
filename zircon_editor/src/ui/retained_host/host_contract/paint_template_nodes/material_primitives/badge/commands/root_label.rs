use super::super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::super::render_commands::HostPaintCommand;
use super::super::geometry::badge_root_text_frame;
use super::super::labels::badge_root_label;
use super::super::style::badge_root_text_color;
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_badge_root_label(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let label = badge_root_label(node);
    if label.is_empty() {
        return;
    }
    let text_frame = badge_root_text_frame(node, rect, &label);
    commands.push(HostPaintCommand::text(
        text_frame.rect,
        Some(clip.clone()),
        order,
        label,
        badge_root_text_color(node),
        text_frame.font_size,
        text_frame.line_height,
        UiTextRunPaintStyle::default(),
        opacity,
    ));
}

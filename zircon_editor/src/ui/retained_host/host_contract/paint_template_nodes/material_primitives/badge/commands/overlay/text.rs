use super::super::super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::super::super::render_commands::HostPaintCommand;
use super::super::super::geometry::badge_overlay_text_frame;
use super::super::super::style::badge_overlay_text_color;
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_badge_overlay_text(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    display: &str,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let text_frame = badge_overlay_text_frame(display, rect);
    commands.push(HostPaintCommand::text(
        text_frame.rect,
        Some(clip.clone()),
        order,
        display.to_string(),
        badge_overlay_text_color(node),
        text_frame.font_size,
        text_frame.line_height,
        UiTextRunPaintStyle::default(),
        opacity,
    ));
}

use super::super::super::data::FrameRect;
use super::super::render_commands::HostPaintCommand;
use super::metrics::tooltip_metrics;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_tooltip_surface(
    commands: &mut Vec<HostPaintCommand>,
    bubble: &FrameRect,
    clip: &FrameRect,
    order: i32,
    shadow: [u8; 4],
    surface: [u8; 4],
    border: [u8; 4],
    opacity: f32,
) {
    let metrics = tooltip_metrics();
    commands.push(HostPaintCommand::quad(
        FrameRect {
            x: bubble.x,
            y: bubble.y + metrics.shadow_offset_y,
            width: bubble.width,
            height: bubble.height,
        },
        Some(clip.clone()),
        order,
        Some(shadow),
        None,
        0.0,
        metrics.radius,
        opacity,
    ));
    commands.push(HostPaintCommand::quad(
        bubble.clone(),
        Some(clip.clone()),
        order + 1,
        Some(surface),
        Some(border),
        metrics.border_width,
        metrics.radius,
        opacity,
    ));
}

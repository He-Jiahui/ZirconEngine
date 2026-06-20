use super::super::super::data::FrameRect;
use super::super::render_commands::HostPaintCommand;

const TOOLTIP_RADIUS: f32 = 4.0;
const TOOLTIP_BORDER_WIDTH: f32 = 1.0;
const TOOLTIP_SHADOW_OFFSET_Y: f32 = 8.0;

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
    commands.push(HostPaintCommand::quad(
        FrameRect {
            x: bubble.x,
            y: bubble.y + TOOLTIP_SHADOW_OFFSET_Y,
            width: bubble.width,
            height: bubble.height,
        },
        Some(clip.clone()),
        order,
        Some(shadow),
        None,
        0.0,
        TOOLTIP_RADIUS,
        opacity,
    ));
    commands.push(HostPaintCommand::quad(
        bubble.clone(),
        Some(clip.clone()),
        order + 1,
        Some(surface),
        Some(border),
        TOOLTIP_BORDER_WIDTH,
        TOOLTIP_RADIUS,
        opacity,
    ));
}

use super::super::super::data::FrameRect;
use super::super::render_commands::HostPaintCommand;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn template_corner_radius_from_rect(
    rect: &FrameRect,
) -> f32 {
    (rect.height * 0.08).clamp(0.0, 4.0)
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_rect_line(
    commands: &mut Vec<HostPaintCommand>,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    commands.push(HostPaintCommand::quad(
        FrameRect {
            x,
            y,
            width: width.max(1.0),
            height: height.max(1.0),
        },
        Some(clip.clone()),
        order,
        Some(color),
        None,
        0.0,
        0.0,
        opacity,
    ));
}

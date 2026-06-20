use super::super::super::render_commands::HostPaintCommand;
use super::super::layout::TOAST_RADIUS;
use crate::ui::retained_host::host_contract::data::FrameRect;

pub(super) fn push_toast_surface(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    surface: [u8; 4],
    border: [u8; 4],
    opacity: f32,
) {
    commands.push(HostPaintCommand::quad(
        rect.clone(),
        Some(clip.clone()),
        order,
        Some(surface),
        Some(border),
        1.0,
        TOAST_RADIUS,
        opacity,
    ));
}

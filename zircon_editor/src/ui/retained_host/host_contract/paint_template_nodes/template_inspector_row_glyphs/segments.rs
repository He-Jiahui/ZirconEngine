use super::super::super::data::FrameRect;
use super::super::render_commands::HostPaintCommand;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_inspector_segments(
    commands: &mut Vec<HostPaintCommand>,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
    segments: &[FrameRect],
    radius: f32,
) {
    for part in segments {
        commands.push(HostPaintCommand::quad(
            part.clone(),
            Some(clip.clone()),
            order,
            Some(color),
            None,
            0.0,
            radius,
            opacity,
        ));
    }
}

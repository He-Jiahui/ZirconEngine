use super::super::super::data::FrameRect;
use super::super::render_commands::HostPaintCommand;

const CYAN_GLOW: [u8; 4] = [34, 193, 203, 56];

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_selection_glow(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    commands.push(HostPaintCommand::quad(
        FrameRect {
            x: rect.x - 2.0,
            y: rect.y - 2.0,
            width: rect.width + 4.0,
            height: rect.height + 4.0,
        },
        Some(clip.clone()),
        order,
        Some(CYAN_GLOW),
        None,
        0.0,
        3.0,
        opacity,
    ));
}

use super::super::super::data::FrameRect;
use super::super::render_commands::HostPaintCommand;

const GIZMO_CUBE: [u8; 4] = [49, 93, 159, 255];
const GIZMO_CUBE_LIGHT: [u8; 4] = [111, 159, 220, 176];
const GIZMO_CUBE_DARK: [u8; 4] = [27, 58, 104, 140];
const GIZMO_Y_ROD: [u8; 4] = [88, 208, 94, 255];

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_gizmo_center(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    commands.push(HostPaintCommand::quad(
        FrameRect {
            x: rect.x + rect.width * 0.5 - 1.0,
            y: rect.y - 28.0,
            width: 2.0,
            height: 28.0,
        },
        Some(clip.clone()),
        order,
        Some(GIZMO_Y_ROD),
        None,
        0.0,
        1.0,
        opacity,
    ));
    commands.push(HostPaintCommand::quad(
        rect.clone(),
        Some(clip.clone()),
        order + 1,
        Some(GIZMO_CUBE),
        None,
        0.0,
        2.0,
        opacity,
    ));
    commands.push(HostPaintCommand::quad(
        FrameRect {
            x: rect.x,
            y: rect.y,
            width: rect.width,
            height: (rect.height * 0.42).max(1.0),
        },
        Some(clip.clone()),
        order + 2,
        Some(GIZMO_CUBE_LIGHT),
        None,
        0.0,
        2.0,
        opacity,
    ));
    commands.push(HostPaintCommand::quad(
        FrameRect {
            x: rect.x + rect.width * 0.66,
            y: rect.y,
            width: (rect.width * 0.34).max(1.0),
            height: rect.height,
        },
        Some(clip.clone()),
        order + 3,
        Some(GIZMO_CUBE_DARK),
        None,
        0.0,
        2.0,
        opacity,
    ));
}

use super::super::data::{FrameRect, TemplatePaneNodeData};
use super::render_commands::HostPaintCommand;
use super::template_style::text_color;
use super::template_style_color::resolved_style_color;
use super::template_viewport_scene_structure::push_base_surface;

const CYAN_GLOW: [u8; 4] = [34, 193, 203, 56];
const AXIS_X: [u8; 4] = [239, 73, 63, 255];
const AXIS_Y: [u8; 4] = [88, 208, 94, 255];
const AXIS_Z: [u8; 4] = [57, 144, 255, 255];
const AXIS_GLOW: [u8; 4] = [34, 193, 203, 64];
const GIZMO_CUBE: [u8; 4] = [49, 93, 159, 255];
const GIZMO_CUBE_LIGHT: [u8; 4] = [111, 159, 220, 176];
const GIZMO_CUBE_DARK: [u8; 4] = [27, 58, 104, 140];
const GIZMO_Y_ROD: [u8; 4] = [88, 208, 94, 255];

pub(super) fn push_selection_glow(
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

pub(super) fn push_axis_line(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let color = axis_color(node);
    let glow_rect = if rect.width >= rect.height {
        FrameRect {
            x: rect.x - 1.0,
            y: rect.y - 2.0,
            width: rect.width + 2.0,
            height: rect.height + 4.0,
        }
    } else {
        FrameRect {
            x: rect.x - 2.0,
            y: rect.y - 1.0,
            width: rect.width + 4.0,
            height: rect.height + 2.0,
        }
    };
    commands.push(HostPaintCommand::quad(
        glow_rect,
        Some(clip.clone()),
        order,
        Some(axis_glow(color)),
        None,
        0.0,
        4.0,
        opacity,
    ));
    commands.push(HostPaintCommand::quad(
        rect.clone(),
        Some(clip.clone()),
        order + 1,
        Some(color),
        None,
        0.0,
        2.0,
        opacity,
    ));
    push_axis_cap(commands, rect, clip, order + 2, color, opacity);
}

pub(super) fn push_axis_origin(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    commands.push(HostPaintCommand::quad(
        FrameRect {
            x: rect.x - 3.0,
            y: rect.y - 3.0,
            width: rect.width + 6.0,
            height: rect.height + 6.0,
        },
        Some(clip.clone()),
        order,
        Some(AXIS_GLOW),
        None,
        0.0,
        8.0,
        opacity,
    ));
    push_base_surface(commands, node, rect, clip, order + 1, opacity);
}

pub(super) fn push_gizmo_center(
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

fn axis_color(node: &TemplatePaneNodeData) -> [u8; 4] {
    if let Some(color) = declared_surface_color(node) {
        return color;
    }
    match node.control_id.as_str() {
        id if id.contains("AxisX") => AXIS_X,
        id if id.contains("AxisY") => AXIS_Y,
        id if id.contains("AxisZ") => AXIS_Z,
        _ => text_color(node),
    }
}

fn declared_surface_color(node: &TemplatePaneNodeData) -> Option<[u8; 4]> {
    resolved_style_color(node.button_style.element.background_color.as_ref())
        .filter(|color| color[3] > 0)
}

fn axis_glow(color: [u8; 4]) -> [u8; 4] {
    [color[0], color[1], color[2], 58]
}

fn push_axis_cap(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    let cap_size = rect.height.max(rect.width.min(5.0)).max(3.0);
    let cap = if rect.width >= rect.height {
        FrameRect {
            x: rect.x + rect.width - cap_size,
            y: rect.y + (rect.height - cap_size) * 0.5,
            width: cap_size,
            height: cap_size,
        }
    } else {
        FrameRect {
            x: rect.x + (rect.width - cap_size) * 0.5,
            y: rect.y,
            width: cap_size,
            height: cap_size,
        }
    };
    commands.push(HostPaintCommand::quad(
        cap,
        Some(clip.clone()),
        order,
        Some(color),
        None,
        0.0,
        2.0,
        opacity,
    ));
}

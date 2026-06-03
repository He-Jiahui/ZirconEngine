use super::super::data::{FrameRect, TemplatePaneNodeData};
use super::render_commands::HostPaintCommand;
use super::template_style::surface_color;
use super::template_viewport_scene_structure::push_base_surface;

const GRID_GLOW: [u8; 4] = [134, 161, 167, 28];
const GRID_MAJOR_GLOW: [u8; 4] = [162, 188, 192, 42];
const PANEL_INSET_LIGHT: [u8; 4] = [157, 178, 184, 26];
const PANEL_INSET_SHADOW: [u8; 4] = [0, 0, 0, 72];

pub(super) fn push_floor_grid_line(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let major = node.control_id.contains("H2")
        || node.control_id.contains("H4")
        || node.control_id.contains("V2")
        || node.control_id.contains("V5");
    let glow_color = if major { GRID_MAJOR_GLOW } else { GRID_GLOW };
    let glow_rect = if rect.width >= rect.height {
        FrameRect {
            x: rect.x,
            y: rect.y - 1.0,
            width: rect.width,
            height: rect.height + 2.0,
        }
    } else {
        FrameRect {
            x: rect.x - 1.0,
            y: rect.y,
            width: rect.width + 2.0,
            height: rect.height,
        }
    };
    commands.push(HostPaintCommand::quad(
        glow_rect,
        Some(clip.clone()),
        order,
        Some(glow_color),
        None,
        0.0,
        0.0,
        opacity,
    ));
    commands.push(HostPaintCommand::quad(
        rect.clone(),
        Some(clip.clone()),
        order + 1,
        Some(surface_color(node)),
        None,
        0.0,
        0.0,
        opacity,
    ));
}

pub(super) fn push_floor_panel_detail(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    push_base_surface(commands, node, rect, clip, order, opacity);
    push_inset_rect(
        commands,
        rect,
        clip,
        order + 1,
        PANEL_INSET_LIGHT,
        4.0,
        opacity,
    );
    commands.push(HostPaintCommand::quad(
        FrameRect {
            x: rect.x + 1.0,
            y: rect.y + rect.height - 2.0,
            width: (rect.width - 2.0).max(1.0),
            height: 1.0,
        },
        Some(clip.clone()),
        order + 2,
        Some(PANEL_INSET_SHADOW),
        None,
        0.0,
        0.0,
        opacity,
    ));
}

pub(super) fn push_floor_seam(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let color = surface_color(node);
    commands.push(HostPaintCommand::quad(
        FrameRect {
            x: rect.x - 1.0,
            y: rect.y,
            width: rect.width + 2.0,
            height: rect.height,
        },
        Some(clip.clone()),
        order,
        Some(color_with_alpha_factor(color, 0.38)),
        None,
        0.0,
        0.0,
        opacity,
    ));
    commands.push(HostPaintCommand::quad(
        rect.clone(),
        Some(clip.clone()),
        order + 1,
        Some(color),
        None,
        0.0,
        0.0,
        opacity,
    ));
}

fn push_inset_rect(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    inset: f32,
    opacity: f32,
) {
    let x = rect.x + inset;
    let y = rect.y + inset;
    let width = (rect.width - inset * 2.0).max(1.0);
    let height = (rect.height - inset * 2.0).max(1.0);
    commands.push(HostPaintCommand::quad(
        FrameRect {
            x,
            y,
            width,
            height: 1.0,
        },
        Some(clip.clone()),
        order,
        Some(color),
        None,
        0.0,
        0.0,
        opacity,
    ));
    commands.push(HostPaintCommand::quad(
        FrameRect {
            x,
            y: y + height - 1.0,
            width,
            height: 1.0,
        },
        Some(clip.clone()),
        order + 1,
        Some(color),
        None,
        0.0,
        0.0,
        opacity,
    ));
    commands.push(HostPaintCommand::quad(
        FrameRect {
            x,
            y,
            width: 1.0,
            height,
        },
        Some(clip.clone()),
        order + 2,
        Some(color),
        None,
        0.0,
        0.0,
        opacity,
    ));
    commands.push(HostPaintCommand::quad(
        FrameRect {
            x: x + width - 1.0,
            y,
            width: 1.0,
            height,
        },
        Some(clip.clone()),
        order + 3,
        Some(color),
        None,
        0.0,
        0.0,
        opacity,
    ));
}

fn color_with_alpha_factor(mut color: [u8; 4], factor: f32) -> [u8; 4] {
    color[3] = ((color[3] as f32) * factor).round().clamp(0.0, 255.0) as u8;
    color
}

use super::super::data::{FrameRect, TemplatePaneNodeData};
use super::render_commands::HostPaintCommand;
use super::template_style::{surface_color, template_corner_radius};
use super::template_viewport_scene_structure::push_base_surface;

const WALL_LIGHT_CORE: [u8; 4] = [232, 244, 246, 172];
const WALL_LIGHT_HALO: [u8; 4] = [163, 192, 203, 64];
const BEACON_CORE: [u8; 4] = [255, 177, 91, 190];
const BEACON_HALO: [u8; 4] = [225, 139, 70, 84];

pub(super) fn push_soft_light(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let color = surface_color(node);
    push_expanded_layer(
        commands,
        rect,
        clip,
        order,
        color_with_alpha_factor(color, 0.34),
        10.0,
        6.0,
        rect.height * 0.48,
        opacity,
    );
    push_inset_layer(
        commands,
        rect,
        clip,
        order + 1,
        color_with_alpha_factor(color, 0.58),
        8.0,
        9.0,
        rect.height * 0.42,
        opacity,
    );
    commands.push(HostPaintCommand::quad(
        FrameRect {
            x: rect.x + rect.width * 0.18,
            y: rect.y + rect.height * 0.36,
            width: (rect.width * 0.64).max(1.0),
            height: (rect.height * 0.24).max(1.0),
        },
        Some(clip.clone()),
        order + 2,
        Some(color_with_alpha_factor(color, 0.82)),
        None,
        0.0,
        (rect.height * 0.16).max(6.0),
        opacity,
    ));
}

pub(super) fn push_soft_shadow(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let color = surface_color(node);
    push_expanded_layer(
        commands,
        rect,
        clip,
        order,
        color_with_alpha_factor(color, 0.44),
        8.0,
        5.0,
        rect.height * 0.42,
        opacity,
    );
    push_inset_layer(
        commands,
        rect,
        clip,
        order + 1,
        color_with_alpha_factor(color, 0.68),
        6.0,
        7.0,
        rect.height * 0.36,
        opacity,
    );
    commands.push(HostPaintCommand::quad(
        FrameRect {
            x: rect.x + rect.width * 0.22,
            y: rect.y + rect.height * 0.40,
            width: (rect.width * 0.56).max(1.0),
            height: (rect.height * 0.28).max(1.0),
        },
        Some(clip.clone()),
        order + 2,
        Some(color_with_alpha_factor(color, 0.86)),
        None,
        0.0,
        (rect.height * 0.14).max(5.0),
        opacity,
    ));
}

pub(super) fn push_floor_reflection(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let color = surface_color(node);
    push_expanded_layer(
        commands,
        rect,
        clip,
        order,
        color_with_alpha_factor(color, 0.30),
        16.0,
        2.0,
        rect.height * 0.44,
        opacity,
    );
    commands.push(HostPaintCommand::quad(
        FrameRect {
            x: rect.x + rect.width * 0.30,
            y: rect.y + rect.height * 0.12,
            width: (rect.width * 0.40).max(1.0),
            height: (rect.height * 0.76).max(1.0),
        },
        Some(clip.clone()),
        order + 1,
        Some(color_with_alpha_factor(color, 0.76)),
        None,
        0.0,
        rect.height * 0.36,
        opacity,
    ));
    commands.push(HostPaintCommand::quad(
        FrameRect {
            x: rect.x + rect.width * 0.42,
            y: rect.y + rect.height * 0.06,
            width: (rect.width * 0.16).max(1.0),
            height: (rect.height * 0.88).max(1.0),
        },
        Some(clip.clone()),
        order + 2,
        Some(color_with_alpha_factor(color, 0.94)),
        None,
        0.0,
        rect.height * 0.30,
        opacity,
    ));
}

pub(super) fn push_wall_light(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let color = surface_color(node);
    push_expanded_layer(
        commands,
        rect,
        clip,
        order,
        WALL_LIGHT_HALO,
        8.0,
        10.0,
        14.0,
        opacity,
    );
    commands.push(HostPaintCommand::quad(
        rect.clone(),
        Some(clip.clone()),
        order + 1,
        Some(color_with_alpha_factor(color, 0.74)),
        None,
        0.0,
        template_corner_radius(node),
        opacity,
    ));
    commands.push(HostPaintCommand::quad(
        FrameRect {
            x: rect.x + 3.0,
            y: rect.y + 1.0,
            width: (rect.width - 6.0).max(1.0),
            height: (rect.height * 0.36).max(1.0),
        },
        Some(clip.clone()),
        order + 2,
        Some(WALL_LIGHT_CORE),
        None,
        0.0,
        3.0,
        opacity,
    ));
}

pub(super) fn push_beacon(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    push_expanded_layer(
        commands,
        rect,
        clip,
        order,
        BEACON_HALO,
        8.0,
        4.0,
        8.0,
        opacity,
    );
    push_base_surface(commands, node, rect, clip, order + 1, opacity);
    commands.push(HostPaintCommand::quad(
        FrameRect {
            x: rect.x + 2.0,
            y: rect.y + 4.0,
            width: (rect.width - 4.0).max(1.0),
            height: (rect.height - 8.0).max(1.0),
        },
        Some(clip.clone()),
        order + 2,
        Some(BEACON_CORE),
        None,
        0.0,
        1.0,
        opacity,
    ));
}

fn push_expanded_layer(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    expand_x: f32,
    expand_y: f32,
    radius: f32,
    opacity: f32,
) {
    commands.push(HostPaintCommand::quad(
        FrameRect {
            x: rect.x - expand_x,
            y: rect.y - expand_y,
            width: rect.width + expand_x * 2.0,
            height: rect.height + expand_y * 2.0,
        },
        Some(clip.clone()),
        order,
        Some(color),
        None,
        0.0,
        radius,
        opacity,
    ));
}

fn push_inset_layer(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    inset_x: f32,
    inset_y: f32,
    radius: f32,
    opacity: f32,
) {
    commands.push(HostPaintCommand::quad(
        FrameRect {
            x: rect.x + inset_x,
            y: rect.y + inset_y,
            width: (rect.width - inset_x * 2.0).max(1.0),
            height: (rect.height - inset_y * 2.0).max(1.0),
        },
        Some(clip.clone()),
        order,
        Some(color),
        None,
        0.0,
        radius,
        opacity,
    ));
}

fn color_with_alpha_factor(mut color: [u8; 4], factor: f32) -> [u8; 4] {
    color[3] = ((color[3] as f32) * factor).round().clamp(0.0, 255.0) as u8;
    color
}

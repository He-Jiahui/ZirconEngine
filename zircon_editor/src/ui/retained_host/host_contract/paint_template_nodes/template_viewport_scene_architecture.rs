use super::super::data::{FrameRect, TemplatePaneNodeData};
use super::render_commands::HostPaintCommand;
use super::template_style::{border_color, surface_color, template_corner_radius};
use super::template_viewport_scene_structure::push_base_surface;

const PANEL_INSET_SHADOW: [u8; 4] = [0, 0, 0, 72];
const DOOR_INSET_LIGHT: [u8; 4] = [192, 211, 218, 46];
const DOOR_INSET_SHADOW: [u8; 4] = [0, 0, 0, 94];
const WARM_COLUMN_EDGE: [u8; 4] = [170, 106, 54, 54];

pub(super) fn push_side_panel_detail(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    push_base_surface(commands, node, rect, clip, order, opacity);
    let line_color = color_with_alpha_factor(border_color(node), 1.75);
    for y in [rect.y + 36.0, rect.y + 78.0, rect.y + 126.0] {
        commands.push(HostPaintCommand::quad(
            FrameRect {
                x: rect.x + 12.0,
                y,
                width: (rect.width - 24.0).max(1.0),
                height: 1.0,
            },
            Some(clip.clone()),
            order + 1,
            Some(line_color),
            None,
            0.0,
            0.0,
            opacity,
        ));
    }
}

pub(super) fn push_side_stairs(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    commands.push(HostPaintCommand::quad(
        rect.clone(),
        Some(clip.clone()),
        order,
        Some(color_with_alpha_factor(surface_color(node), 0.26)),
        Some(color_with_alpha_factor(border_color(node), 0.72)),
        1.0,
        template_corner_radius(node),
        opacity,
    ));
    let step_color = color_with_alpha_factor(surface_color(node), 1.55);
    let mut y = rect.y + 10.0;
    let mut inset = 4.0;
    for step in 0..5 {
        commands.push(HostPaintCommand::quad(
            FrameRect {
                x: rect.x + inset,
                y,
                width: (rect.width - inset - 10.0).max(1.0),
                height: 2.0,
            },
            Some(clip.clone()),
            order + 1 + step as i32,
            Some(step_color),
            None,
            0.0,
            0.0,
            opacity,
        ));
        y += 13.0;
        inset += 8.0;
    }
}

pub(super) fn push_wall_detail_lines(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    commands.push(HostPaintCommand::quad(
        rect.clone(),
        Some(clip.clone()),
        order,
        Some(color_with_alpha_factor(surface_color(node), 0.16)),
        Some(color_with_alpha_factor(border_color(node), 0.52)),
        1.0,
        template_corner_radius(node),
        opacity,
    ));
    let line_color = color_with_alpha_factor(surface_color(node), 1.45);
    for (index, y_factor) in [0.20_f32, 0.38, 0.56, 0.74].into_iter().enumerate() {
        commands.push(HostPaintCommand::quad(
            FrameRect {
                x: rect.x + rect.width * 0.12,
                y: rect.y + rect.height * y_factor,
                width: (rect.width * 0.76).max(1.0),
                height: 2.0,
            },
            Some(clip.clone()),
            order + 1 + index as i32,
            Some(line_color),
            None,
            0.0,
            0.0,
            opacity,
        ));
    }
    for x_factor in [0.24_f32, 0.50, 0.76] {
        commands.push(HostPaintCommand::quad(
            FrameRect {
                x: rect.x + rect.width * x_factor,
                y: rect.y + rect.height * 0.10,
                width: 1.0,
                height: (rect.height * 0.78).max(1.0),
            },
            Some(clip.clone()),
            order + 6,
            Some(color_with_alpha_factor(line_color, 0.76)),
            None,
            0.0,
            0.0,
            opacity,
        ));
    }
}

pub(super) fn push_back_door(
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
        DOOR_INSET_LIGHT,
        8.0,
        opacity,
    );
    commands.push(HostPaintCommand::quad(
        FrameRect {
            x: rect.x + rect.width * 0.5 - 1.0,
            y: rect.y + 8.0,
            width: 2.0,
            height: (rect.height - 16.0).max(1.0),
        },
        Some(clip.clone()),
        order + 2,
        Some(DOOR_INSET_SHADOW),
        None,
        0.0,
        0.0,
        opacity,
    ));
    commands.push(HostPaintCommand::quad(
        FrameRect {
            x: rect.x + 12.0,
            y: rect.y + rect.height * 0.50,
            width: (rect.width - 24.0).max(1.0),
            height: 2.0,
        },
        Some(clip.clone()),
        order + 3,
        Some(DOOR_INSET_SHADOW),
        None,
        0.0,
        0.0,
        opacity,
    ));
}

pub(super) fn push_door_core(
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
        DOOR_INSET_LIGHT,
        5.0,
        opacity,
    );
    commands.push(HostPaintCommand::quad(
        FrameRect {
            x: rect.x + rect.width * 0.5 - 1.0,
            y: rect.y + 5.0,
            width: 2.0,
            height: (rect.height - 10.0).max(1.0),
        },
        Some(clip.clone()),
        order + 2,
        Some(DOOR_INSET_SHADOW),
        None,
        0.0,
        0.0,
        opacity,
    ));
}

pub(super) fn push_wall_column(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    push_base_surface(commands, node, rect, clip, order, opacity);
    commands.push(HostPaintCommand::quad(
        FrameRect {
            x: rect.x + 3.0,
            y: rect.y + 1.0,
            width: 3.0,
            height: (rect.height - 2.0).max(1.0),
        },
        Some(clip.clone()),
        order + 1,
        Some(WARM_COLUMN_EDGE),
        None,
        0.0,
        1.0,
        opacity,
    ));
    commands.push(HostPaintCommand::quad(
        FrameRect {
            x: rect.x + rect.width - 6.0,
            y: rect.y + 1.0,
            width: 3.0,
            height: (rect.height - 2.0).max(1.0),
        },
        Some(clip.clone()),
        order + 2,
        Some(PANEL_INSET_SHADOW),
        None,
        0.0,
        1.0,
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

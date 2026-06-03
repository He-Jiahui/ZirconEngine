use super::super::data::{FrameRect, TemplatePaneNodeData};
use super::render_commands::HostPaintCommand;
use super::template_viewport_scene_structure::push_base_surface;

const BACKDROP_TOP_HAZE: [u8; 4] = [57, 67, 72, 34];
const BACKDROP_SIDE_SHADOW: [u8; 4] = [0, 0, 0, 82];
const CEILING_RIB: [u8; 4] = [96, 112, 118, 30];
const CEILING_BOTTOM_SHADOW: [u8; 4] = [0, 0, 0, 96];
const CEILING_LIGHT_GLINT: [u8; 4] = [214, 228, 233, 42];
const WALL_PANEL_LINE: [u8; 4] = [112, 132, 140, 34];
const WALL_TOP_SHADOW: [u8; 4] = [0, 0, 0, 74];
const WALL_INNER_HAZE: [u8; 4] = [104, 126, 134, 22];
const FLOOR_TOP_SHADOW: [u8; 4] = [0, 0, 0, 86];
const FLOOR_DEPTH_LINE: [u8; 4] = [126, 140, 144, 28];
const FLOOR_WARM_SHEEN: [u8; 4] = [148, 112, 72, 22];
const FLOOR_BOTTOM_SHADOW: [u8; 4] = [0, 0, 0, 62];

pub(super) fn push_backdrop_surface(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    push_base_surface(commands, node, rect, clip, order, opacity);
    push_layer(
        commands,
        FrameRect {
            x: rect.x,
            y: rect.y,
            width: rect.width,
            height: (rect.height * 0.26).max(1.0),
        },
        clip,
        order + 1,
        BACKDROP_TOP_HAZE,
        0.0,
        opacity,
    );
    let side_width = (rect.width * 0.08).clamp(12.0, 84.0);
    push_layer(
        commands,
        FrameRect {
            x: rect.x,
            y: rect.y,
            width: side_width,
            height: rect.height,
        },
        clip,
        order + 2,
        BACKDROP_SIDE_SHADOW,
        0.0,
        opacity,
    );
    push_layer(
        commands,
        FrameRect {
            x: rect.x + rect.width - side_width,
            y: rect.y,
            width: side_width,
            height: rect.height,
        },
        clip,
        order + 3,
        color_with_alpha_factor(BACKDROP_SIDE_SHADOW, 0.76),
        0.0,
        opacity,
    );
}

pub(super) fn push_ceiling_surface(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    push_base_surface(commands, node, rect, clip, order, opacity);
    for x_factor in [0.18_f32, 0.42, 0.68, 0.86] {
        push_layer(
            commands,
            FrameRect {
                x: (rect.x + rect.width * x_factor).round(),
                y: rect.y,
                width: 2.0,
                height: rect.height,
            },
            clip,
            order + 1,
            CEILING_RIB,
            0.0,
            opacity,
        );
    }
    push_layer(
        commands,
        FrameRect {
            x: rect.x + rect.width * 0.35,
            y: rect.y + 14.0,
            width: (rect.width * 0.18).max(18.0),
            height: 3.0,
        },
        clip,
        order + 2,
        CEILING_LIGHT_GLINT,
        2.0,
        opacity,
    );
    push_layer(
        commands,
        FrameRect {
            x: rect.x,
            y: rect.y + rect.height - 10.0,
            width: rect.width,
            height: 10.0,
        },
        clip,
        order + 3,
        CEILING_BOTTOM_SHADOW,
        0.0,
        opacity,
    );
}

pub(super) fn push_back_wall_surface(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    push_base_surface(commands, node, rect, clip, order, opacity);
    push_layer(
        commands,
        FrameRect {
            x: rect.x,
            y: rect.y,
            width: rect.width,
            height: (rect.height * 0.18).max(1.0),
        },
        clip,
        order + 1,
        WALL_TOP_SHADOW,
        0.0,
        opacity,
    );
    for (index, y_factor) in [0.34_f32, 0.68].into_iter().enumerate() {
        push_layer(
            commands,
            FrameRect {
                x: rect.x + rect.width * 0.08,
                y: (rect.y + rect.height * y_factor).round(),
                width: (rect.width * 0.84).max(1.0),
                height: 1.0,
            },
            clip,
            order + 2 + index as i32,
            WALL_PANEL_LINE,
            0.0,
            opacity,
        );
    }
    for (index, x_factor) in [0.24_f32, 0.50, 0.76].into_iter().enumerate() {
        push_layer(
            commands,
            FrameRect {
                x: (rect.x + rect.width * x_factor).round(),
                y: rect.y + rect.height * 0.16,
                width: 1.0,
                height: (rect.height * 0.70).max(1.0),
            },
            clip,
            order + 4 + index as i32,
            WALL_PANEL_LINE,
            0.0,
            opacity,
        );
    }
    push_layer(
        commands,
        FrameRect {
            x: rect.x + rect.width * 0.36,
            y: rect.y + rect.height * 0.20,
            width: (rect.width * 0.28).max(1.0),
            height: (rect.height * 0.52).max(1.0),
        },
        clip,
        order + 7,
        WALL_INNER_HAZE,
        8.0,
        opacity,
    );
}

pub(super) fn push_floor_surface(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    push_base_surface(commands, node, rect, clip, order, opacity);
    push_layer(
        commands,
        FrameRect {
            x: rect.x,
            y: rect.y,
            width: rect.width,
            height: (rect.height * 0.15).max(1.0),
        },
        clip,
        order + 1,
        FLOOR_TOP_SHADOW,
        0.0,
        opacity,
    );
    for (index, y_factor) in [0.28_f32, 0.56, 0.78].into_iter().enumerate() {
        push_layer(
            commands,
            FrameRect {
                x: rect.x + rect.width * 0.06,
                y: (rect.y + rect.height * y_factor).round(),
                width: (rect.width * 0.88).max(1.0),
                height: 1.0,
            },
            clip,
            order + 2 + index as i32,
            FLOOR_DEPTH_LINE,
            0.0,
            opacity,
        );
    }
    for (index, x_factor) in [0.30_f32, 0.52, 0.74].into_iter().enumerate() {
        push_layer(
            commands,
            FrameRect {
                x: (rect.x + rect.width * x_factor).round(),
                y: rect.y + rect.height * 0.18,
                width: 1.0,
                height: (rect.height * 0.72).max(1.0),
            },
            clip,
            order + 5 + index as i32,
            color_with_alpha_factor(FLOOR_DEPTH_LINE, 0.76),
            0.0,
            opacity,
        );
    }
    push_layer(
        commands,
        FrameRect {
            x: rect.x + rect.width * 0.42,
            y: rect.y + rect.height * 0.22,
            width: (rect.width * 0.22).max(1.0),
            height: (rect.height * 0.66).max(1.0),
        },
        clip,
        order + 8,
        FLOOR_WARM_SHEEN,
        14.0,
        opacity,
    );
    push_layer(
        commands,
        FrameRect {
            x: rect.x,
            y: rect.y + rect.height - 18.0,
            width: rect.width,
            height: 18.0,
        },
        clip,
        order + 9,
        FLOOR_BOTTOM_SHADOW,
        0.0,
        opacity,
    );
}

fn push_layer(
    commands: &mut Vec<HostPaintCommand>,
    rect: FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    radius: f32,
    opacity: f32,
) {
    commands.push(HostPaintCommand::quad(
        rect,
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

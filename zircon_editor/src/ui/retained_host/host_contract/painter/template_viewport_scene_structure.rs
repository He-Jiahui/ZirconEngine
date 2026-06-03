use super::super::data::{FrameRect, TemplatePaneNodeData};
use super::render_commands::HostPaintCommand;
use super::template_style::{
    border_color, resolved_style_color, surface_color, template_border_width,
    template_corner_radius, text_color,
};

const CYAN_GLOW: [u8; 4] = [34, 193, 203, 56];
const AXIS_X: [u8; 4] = [239, 73, 63, 255];
const AXIS_Y: [u8; 4] = [88, 208, 94, 255];
const AXIS_Z: [u8; 4] = [57, 144, 255, 255];
const AXIS_GLOW: [u8; 4] = [34, 193, 203, 64];
const GRATE_DARK: [u8; 4] = [0, 0, 0, 132];
const GRATE_WARM: [u8; 4] = [112, 96, 78, 46];
const GRATE_EDGE_LIGHT: [u8; 4] = [180, 186, 180, 31];
const CARGO_HIGHLIGHT: [u8; 4] = [255, 255, 255, 11];
const CARGO_INSET_SHADOW: [u8; 4] = [0, 0, 0, 62];
const CARGO_INNER_LINE: [u8; 4] = [148, 160, 162, 48];
const CARGO_INNER_CORNER: [u8; 4] = [190, 205, 208, 64];
const PROP_TOP_HIGHLIGHT: [u8; 4] = [255, 255, 255, 24];
const PROP_EDGE_LIGHT: [u8; 4] = [180, 198, 202, 22];
const PROP_SIDE_SHADOW: [u8; 4] = [0, 0, 0, 72];
const PROP_BOTTOM_SHADOW: [u8; 4] = [0, 0, 0, 54];
const HANDRAIL_POST: [u8; 4] = [179, 113, 48, 107];
const HANDRAIL_BOTTOM: [u8; 4] = [143, 88, 40, 97];
const GIZMO_CUBE: [u8; 4] = [49, 93, 159, 255];
const GIZMO_CUBE_LIGHT: [u8; 4] = [111, 159, 220, 176];
const GIZMO_CUBE_DARK: [u8; 4] = [27, 58, 104, 140];
const GIZMO_Y_ROD: [u8; 4] = [88, 208, 94, 255];

pub(super) fn push_base_surface(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let border_width = template_border_width(node);
    commands.push(HostPaintCommand::quad(
        rect.clone(),
        Some(clip.clone()),
        order,
        Some(surface_color(node)),
        (border_width > 0.0).then(|| border_color(node)),
        border_width,
        template_corner_radius(node),
        opacity,
    ));
}

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

pub(super) fn push_floor_grate_slots(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    commands.push(HostPaintCommand::quad(
        FrameRect {
            x: rect.x + 1.0,
            y: rect.y,
            width: 2.0,
            height: rect.height,
        },
        Some(clip.clone()),
        order,
        Some(GRATE_DARK),
        None,
        0.0,
        0.0,
        opacity,
    ));
    commands.push(HostPaintCommand::quad(
        FrameRect {
            x: rect.x + rect.width - 2.0,
            y: rect.y,
            width: 1.0,
            height: rect.height,
        },
        Some(clip.clone()),
        order,
        Some(GRATE_EDGE_LIGHT),
        None,
        0.0,
        0.0,
        opacity,
    ));

    let mut x = rect.x + 4.0;
    let max_x = rect.x + rect.width - 3.0;
    let mut stripe_order = order + 1;
    while x < max_x {
        commands.push(HostPaintCommand::quad(
            FrameRect {
                x,
                y: rect.y,
                width: 2.0,
                height: rect.height,
            },
            Some(clip.clone()),
            stripe_order,
            Some(GRATE_DARK),
            None,
            0.0,
            0.0,
            opacity,
        ));
        commands.push(HostPaintCommand::quad(
            FrameRect {
                x: x + 2.0,
                y: rect.y,
                width: 3.0,
                height: rect.height,
            },
            Some(clip.clone()),
            stripe_order + 1,
            Some(GRATE_WARM),
            None,
            0.0,
            0.0,
            opacity,
        ));
        x += 8.0;
        stripe_order += 2;
    }
}

pub(super) fn push_cargo_detail(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let mut x = rect.x + 8.0;
    while x < rect.x + rect.width - 4.0 {
        commands.push(HostPaintCommand::quad(
            FrameRect {
                x,
                y: rect.y + 2.0,
                width: 1.0,
                height: (rect.height - 4.0).max(1.0),
            },
            Some(clip.clone()),
            order,
            Some(CARGO_HIGHLIGHT),
            None,
            0.0,
            0.0,
            opacity,
        ));
        x += 28.0;
    }
    commands.push(HostPaintCommand::quad(
        FrameRect {
            x: rect.x + (rect.width * 0.82).max(0.0),
            y: rect.y + 1.0,
            width: (rect.width * 0.18).max(1.0),
            height: (rect.height - 2.0).max(1.0),
        },
        Some(clip.clone()),
        order + 1,
        Some(CARGO_INSET_SHADOW),
        None,
        0.0,
        template_corner_radius_from_rect(rect),
        opacity,
    ));
    commands.push(HostPaintCommand::quad(
        FrameRect {
            x: rect.x + 1.0,
            y: rect.y + (rect.height * 0.82).max(0.0),
            width: (rect.width - 2.0).max(1.0),
            height: (rect.height * 0.18).max(1.0),
        },
        Some(clip.clone()),
        order + 2,
        Some(CARGO_INSET_SHADOW),
        None,
        0.0,
        template_corner_radius_from_rect(rect),
        opacity,
    ));
}

pub(super) fn push_cargo_inner_frame(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    push_rect_line(
        commands,
        rect.x,
        rect.y,
        rect.width,
        1.0,
        clip,
        order,
        CARGO_INNER_CORNER,
        opacity,
    );
    push_rect_line(
        commands,
        rect.x,
        rect.y + rect.height - 1.0,
        rect.width,
        1.0,
        clip,
        order + 1,
        CARGO_INNER_LINE,
        opacity,
    );
    push_rect_line(
        commands,
        rect.x,
        rect.y,
        1.0,
        rect.height,
        clip,
        order + 2,
        CARGO_INNER_CORNER,
        opacity,
    );
    push_rect_line(
        commands,
        rect.x + rect.width - 1.0,
        rect.y,
        1.0,
        rect.height,
        clip,
        order + 3,
        CARGO_INNER_LINE,
        opacity,
    );

    if rect.width >= 48.0 {
        let mut divider_order = order + 4;
        for x_factor in [0.34_f32, 0.66] {
            push_rect_line(
                commands,
                (rect.x + rect.width * x_factor).round(),
                rect.y + 4.0,
                1.0,
                (rect.height - 8.0).max(1.0),
                clip,
                divider_order,
                CARGO_INNER_LINE,
                opacity,
            );
            divider_order += 1;
        }
    }

    if rect.height >= 32.0 {
        push_rect_line(
            commands,
            rect.x + 4.0,
            (rect.y + rect.height * 0.52).round(),
            (rect.width - 8.0).max(1.0),
            1.0,
            clip,
            order + 6,
            CARGO_INNER_LINE,
            opacity,
        );
    }
}

pub(super) fn push_prop_top_detail(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    push_base_surface(commands, node, rect, clip, order, opacity);
    push_rect_line(
        commands,
        rect.x + 2.0,
        rect.y + 2.0,
        (rect.width - 4.0).max(1.0),
        3.0,
        clip,
        order + 1,
        PROP_TOP_HIGHLIGHT,
        opacity,
    );
    push_rect_line(
        commands,
        rect.x + rect.width - 4.0,
        rect.y + 3.0,
        2.0,
        (rect.height - 6.0).max(1.0),
        clip,
        order + 2,
        PROP_SIDE_SHADOW,
        opacity,
    );
    push_rect_line(
        commands,
        rect.x + 2.0,
        rect.y + rect.height - 3.0,
        (rect.width - 4.0).max(1.0),
        2.0,
        clip,
        order + 3,
        PROP_BOTTOM_SHADOW,
        opacity,
    );
}

pub(super) fn push_prop_body_detail(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    push_base_surface(commands, node, rect, clip, order, opacity);
    push_rect_line(
        commands,
        rect.x + 2.0,
        rect.y + 2.0,
        (rect.width - 4.0).max(1.0),
        5.0,
        clip,
        order + 1,
        PROP_TOP_HIGHLIGHT,
        opacity,
    );
    push_rect_line(
        commands,
        rect.x + 3.0,
        rect.y + 8.0,
        2.0,
        (rect.height - 14.0).max(1.0),
        clip,
        order + 2,
        PROP_EDGE_LIGHT,
        opacity,
    );
    push_rect_line(
        commands,
        rect.x + rect.width - 5.0,
        rect.y + 6.0,
        3.0,
        (rect.height - 12.0).max(1.0),
        clip,
        order + 3,
        PROP_SIDE_SHADOW,
        opacity,
    );
    push_rect_line(
        commands,
        rect.x + 2.0,
        rect.y + rect.height - 5.0,
        (rect.width - 4.0).max(1.0),
        4.0,
        clip,
        order + 4,
        PROP_BOTTOM_SHADOW,
        opacity,
    );
}

pub(super) fn push_rack_detail(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let vertical = [0, 0, 0, 112];
    let horizontal = [172, 109, 55, 31];
    let mut x = rect.x + 8.0;
    while x < rect.x + rect.width {
        commands.push(HostPaintCommand::quad(
            FrameRect {
                x,
                y: rect.y,
                width: 2.0,
                height: rect.height,
            },
            Some(clip.clone()),
            order,
            Some(vertical),
            None,
            0.0,
            0.0,
            opacity,
        ));
        x += 28.0;
    }
    let mut y = rect.y + 3.0;
    while y < rect.y + rect.height {
        commands.push(HostPaintCommand::quad(
            FrameRect {
                x: rect.x,
                y,
                width: rect.width,
                height: 2.0,
            },
            Some(clip.clone()),
            order + 1,
            Some(horizontal),
            None,
            0.0,
            0.0,
            opacity,
        ));
        y += 42.0;
    }
}

pub(super) fn push_handrail(
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
            x: rect.x,
            y: rect.y + rect.height + 1.0,
            width: rect.width,
            height: 2.0,
        },
        Some(clip.clone()),
        order + 1,
        Some(HANDRAIL_BOTTOM),
        None,
        0.0,
        0.0,
        opacity,
    ));
    for x in [rect.x + 36.0, rect.x + rect.width - 42.0] {
        commands.push(HostPaintCommand::quad(
            FrameRect {
                x,
                y: rect.y - 3.0,
                width: 4.0,
                height: 56.0,
            },
            Some(clip.clone()),
            order + 2,
            Some(HANDRAIL_POST),
            None,
            0.0,
            1.0,
            opacity,
        ));
    }
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

fn template_corner_radius_from_rect(rect: &FrameRect) -> f32 {
    (rect.height * 0.08).clamp(0.0, 4.0)
}

fn push_rect_line(
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

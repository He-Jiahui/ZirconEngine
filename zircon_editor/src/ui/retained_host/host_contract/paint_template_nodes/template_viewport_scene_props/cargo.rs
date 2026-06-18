use super::super::super::data::FrameRect;
use super::super::render_commands::HostPaintCommand;
use super::primitives::{push_rect_line, template_corner_radius_from_rect};

const CARGO_HIGHLIGHT: [u8; 4] = [255, 255, 255, 11];
const CARGO_INSET_SHADOW: [u8; 4] = [0, 0, 0, 62];
const CARGO_INNER_LINE: [u8; 4] = [148, 160, 162, 48];
const CARGO_INNER_CORNER: [u8; 4] = [190, 205, 208, 64];

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_cargo_detail(
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

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_cargo_inner_frame(
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

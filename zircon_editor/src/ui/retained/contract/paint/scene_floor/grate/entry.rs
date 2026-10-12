use crate::ui::retained_host::host_contract::data::FrameRect;
use crate::ui::retained_host::host_contract::paint_template_nodes::render_commands::HostPaintCommand;

use super::palette::{GRATE_DARK, GRATE_EDGE_LIGHT, GRATE_WARM};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_floor_grate_slots(
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

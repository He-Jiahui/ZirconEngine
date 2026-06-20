use crate::ui::retained_host::host_contract::data::FrameRect;
use crate::ui::retained_host::host_contract::paint_template_nodes::render_commands::HostPaintCommand;

use super::super::primitives::template_corner_radius_from_rect;

const CARGO_HIGHLIGHT: [u8; 4] = [255, 255, 255, 11];
const CARGO_INSET_SHADOW: [u8; 4] = [0, 0, 0, 62];

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

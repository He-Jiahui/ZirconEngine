use crate::ui::retained_host::host_contract::data::FrameRect;
use crate::ui::retained_host::host_contract::paint_template_nodes::render_commands::HostPaintCommand;
use crate::ui::retained_host::host_contract::paint_template_nodes::template_viewport_scene_surfaces::primitives::push_layer;

use super::palette::{FLOOR_BOTTOM_SHADOW, FLOOR_TOP_SHADOW};

pub(super) fn push_floor_top_shadow(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    push_layer(
        commands,
        FrameRect {
            x: rect.x,
            y: rect.y,
            width: rect.width,
            height: (rect.height * 0.15).max(1.0),
        },
        clip,
        order,
        FLOOR_TOP_SHADOW,
        0.0,
        opacity,
    );
}

pub(super) fn push_floor_bottom_shadow(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    push_layer(
        commands,
        FrameRect {
            x: rect.x,
            y: rect.y + rect.height - 18.0,
            width: rect.width,
            height: 18.0,
        },
        clip,
        order,
        FLOOR_BOTTOM_SHADOW,
        0.0,
        opacity,
    );
}

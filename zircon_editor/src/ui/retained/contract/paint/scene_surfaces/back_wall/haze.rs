use crate::ui::retained_host::host_contract::data::FrameRect;
use crate::ui::retained_host::host_contract::paint_template_nodes::render_commands::HostPaintCommand;
use crate::ui::retained_host::host_contract::paint_template_nodes::template_viewport_scene_surfaces::primitives::push_layer;

use super::palette::WALL_INNER_HAZE;

pub(super) fn push_wall_inner_haze(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    push_layer(
        commands,
        FrameRect {
            x: rect.x + rect.width * 0.36,
            y: rect.y + rect.height * 0.20,
            width: (rect.width * 0.28).max(1.0),
            height: (rect.height * 0.52).max(1.0),
        },
        clip,
        order,
        WALL_INNER_HAZE,
        8.0,
        opacity,
    );
}

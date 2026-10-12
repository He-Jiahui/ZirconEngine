use crate::ui::retained_host::host_contract::data::FrameRect;
use crate::ui::retained_host::host_contract::paint_template_nodes::render_commands::HostPaintCommand;
use crate::ui::retained_host::host_contract::paint_template_nodes::template_viewport_scene_surfaces::primitives::push_layer;

use super::palette::WALL_TOP_SHADOW;

pub(super) fn push_wall_top_shadow(
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
            height: (rect.height * 0.18).max(1.0),
        },
        clip,
        order,
        WALL_TOP_SHADOW,
        0.0,
        opacity,
    );
}

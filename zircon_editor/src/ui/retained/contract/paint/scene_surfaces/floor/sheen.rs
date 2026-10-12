use crate::ui::retained_host::host_contract::data::FrameRect;
use crate::ui::retained_host::host_contract::paint_template_nodes::render_commands::HostPaintCommand;
use crate::ui::retained_host::host_contract::paint_template_nodes::template_viewport_scene_surfaces::primitives::push_layer;

use super::palette::FLOOR_WARM_SHEEN;

pub(super) fn push_floor_warm_sheen(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    push_layer(
        commands,
        FrameRect {
            x: rect.x + rect.width * 0.42,
            y: rect.y + rect.height * 0.22,
            width: (rect.width * 0.22).max(1.0),
            height: (rect.height * 0.66).max(1.0),
        },
        clip,
        order,
        FLOOR_WARM_SHEEN,
        14.0,
        opacity,
    );
}

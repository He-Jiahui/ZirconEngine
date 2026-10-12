use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::render_commands::HostPaintCommand;
use super::super::template_viewport_scene_structure::push_base_surface;
use super::primitives::{color_with_alpha_factor, push_layer};

const BACKDROP_TOP_HAZE: [u8; 4] = [57, 67, 72, 34];
const BACKDROP_SIDE_SHADOW: [u8; 4] = [0, 0, 0, 82];

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_backdrop_surface(
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

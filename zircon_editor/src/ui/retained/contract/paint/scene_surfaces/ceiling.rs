use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::render_commands::HostPaintCommand;
use super::super::template_viewport_scene_structure::push_base_surface;
use super::primitives::push_layer;

const CEILING_RIB: [u8; 4] = [96, 112, 118, 30];
const CEILING_BOTTOM_SHADOW: [u8; 4] = [0, 0, 0, 96];
const CEILING_LIGHT_GLINT: [u8; 4] = [214, 228, 233, 42];

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_ceiling_surface(
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

use crate::ui::retained_host::host_contract::data::FrameRect;
use crate::ui::retained_host::host_contract::paint_template_nodes::render_commands::HostPaintCommand;
use crate::ui::retained_host::host_contract::paint_template_nodes::template_viewport_scene_surfaces::primitives::{
    color_with_alpha_factor, push_layer,
};

use super::palette::FLOOR_DEPTH_LINE;

pub(super) fn push_floor_depth_lines(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    for (index, y_factor) in [0.28_f32, 0.56, 0.78].into_iter().enumerate() {
        push_layer(
            commands,
            FrameRect {
                x: rect.x + rect.width * 0.06,
                y: (rect.y + rect.height * y_factor).round(),
                width: (rect.width * 0.88).max(1.0),
                height: 1.0,
            },
            clip,
            order + index as i32,
            FLOOR_DEPTH_LINE,
            0.0,
            opacity,
        );
    }
    for (index, x_factor) in [0.30_f32, 0.52, 0.74].into_iter().enumerate() {
        push_layer(
            commands,
            FrameRect {
                x: (rect.x + rect.width * x_factor).round(),
                y: rect.y + rect.height * 0.18,
                width: 1.0,
                height: (rect.height * 0.72).max(1.0),
            },
            clip,
            order + 3 + index as i32,
            color_with_alpha_factor(FLOOR_DEPTH_LINE, 0.76),
            0.0,
            opacity,
        );
    }
}

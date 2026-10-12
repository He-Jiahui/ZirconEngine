use crate::ui::retained_host::host_contract::data::FrameRect;
use crate::ui::retained_host::host_contract::paint_template_nodes::render_commands::HostPaintCommand;
use crate::ui::retained_host::host_contract::paint_template_nodes::template_viewport_scene_surfaces::primitives::push_layer;

use super::palette::WALL_PANEL_LINE;

pub(super) fn push_wall_panel_lines(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    for (index, y_factor) in [0.34_f32, 0.68].into_iter().enumerate() {
        push_layer(
            commands,
            FrameRect {
                x: rect.x + rect.width * 0.08,
                y: (rect.y + rect.height * y_factor).round(),
                width: (rect.width * 0.84).max(1.0),
                height: 1.0,
            },
            clip,
            order + index as i32,
            WALL_PANEL_LINE,
            0.0,
            opacity,
        );
    }
    for (index, x_factor) in [0.24_f32, 0.50, 0.76].into_iter().enumerate() {
        push_layer(
            commands,
            FrameRect {
                x: (rect.x + rect.width * x_factor).round(),
                y: rect.y + rect.height * 0.16,
                width: 1.0,
                height: (rect.height * 0.70).max(1.0),
            },
            clip,
            order + 2 + index as i32,
            WALL_PANEL_LINE,
            0.0,
            opacity,
        );
    }
}

use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::render_commands::HostPaintCommand;
use super::geometry::SampleGridGeometry;
use super::metrics::{AXIS_FONT_SIZE, AXIS_LINE_HEIGHT, TICK_FONT_SIZE, TICK_LINE_HEIGHT};
use super::palette::{AXIS_TEXT, TICK_TEXT};

pub(super) fn push_sample_grid_text(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    geometry: &SampleGridGeometry,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let grid = &node.sample_grid.generation;
    for tick in grid.x_ticks() {
        let x = geometry.x_for_value(tick.value(), grid.x_min(), grid.x_max());
        push_text(
            commands,
            FrameRect {
                x: x - 24.0,
                y: geometry.plot.y + geometry.plot.height + 4.0,
                width: 48.0,
                height: TICK_LINE_HEIGHT,
            },
            clip,
            order + 4,
            tick.label().to_string(),
            TICK_TEXT,
            TICK_FONT_SIZE,
            TICK_LINE_HEIGHT,
            opacity,
        );
    }
    for tick in grid.y_ticks() {
        let y = geometry.y_for_value(tick.value(), grid.y_min(), grid.y_max());
        push_text(
            commands,
            FrameRect {
                x: geometry.outer.x + 3.0,
                y: y - TICK_LINE_HEIGHT * 0.5,
                width: (geometry.plot.x - geometry.outer.x - 7.0).max(1.0),
                height: TICK_LINE_HEIGHT,
            },
            clip,
            order + 4,
            tick.label().to_string(),
            TICK_TEXT,
            TICK_FONT_SIZE,
            TICK_LINE_HEIGHT,
            opacity,
        );
    }

    if !grid.y_axis_label().trim().is_empty() {
        push_text(
            commands,
            FrameRect {
                x: geometry.plot.x,
                y: geometry.outer.y + 4.0,
                width: geometry.plot.width * 0.5,
                height: AXIS_LINE_HEIGHT,
            },
            clip,
            order + 5,
            grid.y_axis_label().to_string(),
            AXIS_TEXT,
            AXIS_FONT_SIZE,
            AXIS_LINE_HEIGHT,
            opacity,
        );
    }
    if !grid.x_axis_label().trim().is_empty() {
        push_text(
            commands,
            FrameRect {
                x: geometry.plot.x + geometry.plot.width * 0.34,
                y: geometry.outer.y + geometry.outer.height - AXIS_LINE_HEIGHT - 3.0,
                width: geometry.plot.width * 0.66,
                height: AXIS_LINE_HEIGHT,
            },
            clip,
            order + 5,
            grid.x_axis_label().to_string(),
            AXIS_TEXT,
            AXIS_FONT_SIZE,
            AXIS_LINE_HEIGHT,
            opacity,
        );
    }
}

pub(super) fn push_text(
    commands: &mut Vec<HostPaintCommand>,
    frame: FrameRect,
    clip: &FrameRect,
    order: i32,
    text: String,
    color: [u8; 4],
    font_size: f32,
    line_height: f32,
    opacity: f32,
) {
    commands.push(HostPaintCommand::text(
        frame,
        Some(clip.clone()),
        order,
        text,
        color,
        font_size,
        line_height,
        UiTextRunPaintStyle::default(),
        opacity,
    ));
}

use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::render_commands::HostPaintCommand;
use super::geometry::SampleGridGeometry;
use super::metrics::{
    POINT_RADIUS, SAMPLE_LABEL_CHARACTER_WIDTH, SAMPLE_LABEL_HEIGHT, SAMPLE_LABEL_MIN_WIDTH,
    SAMPLE_LABEL_OFFSET_X, SAMPLE_LABEL_OFFSET_Y, SELECTED_POINT_RADIUS, TICK_FONT_SIZE,
    TICK_LINE_HEIGHT,
};
use super::palette::{
    POINT, POINT_CENTER, SELECTED_HALO, SELECTED_LABEL_SURFACE, SELECTED_LABEL_TEXT, SELECTED_POINT,
};
use super::text::push_text;

pub(super) fn push_sample_points(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    geometry: &SampleGridGeometry,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    for row in 0..node.sample_grid.points.row_count() {
        let Some(point) = node.sample_grid.points.row_data(row) else {
            continue;
        };
        let x = geometry.point_x_for_value(point.x, node.sample_grid.x_min, node.sample_grid.x_max);
        let y = geometry.point_y_for_value(point.y, node.sample_grid.y_min, node.sample_grid.y_max);
        if point.selected {
            push_diamond(
                commands,
                x,
                y,
                SELECTED_POINT_RADIUS,
                SELECTED_HALO,
                clip,
                order + 6,
                opacity,
            );
        }
        push_diamond(
            commands,
            x,
            y,
            POINT_RADIUS,
            if point.selected {
                SELECTED_POINT
            } else {
                POINT
            },
            clip,
            order + 7,
            opacity,
        );
        push_diamond(commands, x, y, 1, POINT_CENTER, clip, order + 8, opacity);

        if point.selected && !point.label.trim().is_empty() {
            let label_width = ((point.label.chars().count() as f32 * SAMPLE_LABEL_CHARACTER_WIDTH)
                + 12.0)
                .max(SAMPLE_LABEL_MIN_WIDTH)
                .min(geometry.plot.width * 0.6);
            let label_x = (x + SAMPLE_LABEL_OFFSET_X)
                .min(geometry.plot.x + geometry.plot.width - label_width - 2.0)
                .max(geometry.plot.x + 2.0);
            let label_y = (y + SAMPLE_LABEL_OFFSET_Y)
                .max(geometry.plot.y + 2.0)
                .min(geometry.plot.y + geometry.plot.height - SAMPLE_LABEL_HEIGHT - 2.0);
            let label_frame = FrameRect {
                x: label_x,
                y: label_y,
                width: label_width,
                height: SAMPLE_LABEL_HEIGHT,
            };
            commands.push(HostPaintCommand::quad(
                label_frame.clone(),
                Some(clip.clone()),
                order + 9,
                Some(SELECTED_LABEL_SURFACE),
                Some(SELECTED_POINT),
                1.0,
                2.0,
                opacity,
            ));
            push_text(
                commands,
                FrameRect {
                    x: label_frame.x + 5.0,
                    y: label_frame.y + 2.0,
                    width: (label_frame.width - 10.0).max(1.0),
                    height: TICK_LINE_HEIGHT,
                },
                clip,
                order + 10,
                point.label.to_string(),
                SELECTED_LABEL_TEXT,
                TICK_FONT_SIZE,
                TICK_LINE_HEIGHT,
                opacity,
            );
        }
    }
}

fn push_diamond(
    commands: &mut Vec<HostPaintCommand>,
    x: f32,
    y: f32,
    radius: i32,
    color: [u8; 4],
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    for offset in -radius..=radius {
        let half_width = radius - offset.abs();
        commands.push(HostPaintCommand::quad(
            FrameRect {
                x: x - half_width as f32,
                y: y + offset as f32,
                width: (half_width * 2 + 1) as f32,
                height: 1.0,
            },
            Some(clip.clone()),
            order,
            Some(color),
            None,
            0.0,
            0.0,
            opacity,
        ));
    }
}

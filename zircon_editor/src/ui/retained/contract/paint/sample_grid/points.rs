use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::paint_geometry::intersect;
use super::super::super::paint_text::measure_runtime_text_width;
use super::super::render_commands::HostPaintCommand;
use super::super::template_diamond_glyph::push_aa_diamond;
use super::geometry::SampleGridGeometry;
use super::metrics::{
    SampleGridMetrics, SAMPLE_LABEL_HEIGHT, SAMPLE_LABEL_MIN_WIDTH, SAMPLE_LABEL_POINT_GAP,
    TICK_FONT_SIZE, TICK_LINE_HEIGHT,
};
use super::palette::SampleGridPalette;
use super::text::push_text;

pub(super) fn push_sample_points(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    geometry: &SampleGridGeometry,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
    metrics: SampleGridMetrics,
    palette: SampleGridPalette,
) {
    let Some(point_clip) = intersect(clip, &geometry.plot) else {
        return;
    };
    let grid = &node.sample_grid.generation;
    for point in grid.points() {
        let x = geometry.point_x_for_value(point.x(), grid.x_min(), grid.x_max());
        let y = geometry.point_y_for_value(point.y(), grid.y_min(), grid.y_max());
        push_aa_diamond(
            commands,
            x,
            y,
            metrics.point_radius,
            if point.selected() {
                palette.selected_point
            } else {
                palette.point
            },
            &point_clip,
            order + 7,
            opacity,
        );
        push_aa_diamond(
            commands,
            x,
            y,
            metrics.point_interior_radius,
            palette.plot_surface,
            &point_clip,
            order + 8,
            opacity,
        );

        if point.selected() && !point.label().trim().is_empty() {
            let label_width = selected_sample_label_width(point.label(), geometry.plot.width);
            if label_width <= f32::EPSILON || geometry.plot.height < SAMPLE_LABEL_HEIGHT + 4.0 {
                continue;
            }
            let label_x = selected_sample_label_x(x, label_width, &geometry.plot);
            let Some(label_y) = selected_sample_label_y(y, &geometry.plot, metrics.point_radius)
            else {
                continue;
            };
            let label_frame = FrameRect {
                x: label_x,
                y: label_y,
                width: label_width,
                height: SAMPLE_LABEL_HEIGHT,
            };
            commands.push(HostPaintCommand::quad(
                label_frame.clone(),
                Some(point_clip.clone()),
                order + 9,
                Some(palette.selected_label_surface),
                Some(palette.selected_point),
                metrics.selected_label_border_width,
                metrics.selected_label_radius,
                opacity,
            ));
            push_text(
                commands,
                FrameRect {
                    x: label_frame.x + 5.0,
                    y: label_frame.y + 2.0,
                    width: (label_frame.width - 10.0).max(0.0),
                    height: TICK_LINE_HEIGHT,
                },
                &point_clip,
                order + 10,
                point.label().to_string(),
                palette.selected_label_text,
                TICK_FONT_SIZE,
                TICK_LINE_HEIGHT,
                opacity,
            );
        }
    }
}

fn selected_sample_label_width(label: &str, plot_width: f32) -> f32 {
    let available_width = if plot_width.is_finite() {
        plot_width.max(0.0) * 0.6
    } else {
        0.0
    };
    if available_width < SAMPLE_LABEL_MIN_WIDTH {
        return 0.0;
    }
    (measure_runtime_text_width(label, TICK_FONT_SIZE) + 12.0)
        .max(SAMPLE_LABEL_MIN_WIDTH)
        .min(available_width)
}

fn selected_sample_label_x(point_x: f32, label_width: f32, plot: &FrameRect) -> f32 {
    const EDGE_INSET: f32 = 2.0;

    let min_x = plot.x + EDGE_INSET;
    let max_x = (plot.x + plot.width - label_width - EDGE_INSET).max(min_x);
    (point_x - label_width * 0.5).clamp(min_x, max_x)
}

fn selected_sample_label_y(point_y: f32, plot: &FrameRect, point_radius: f32) -> Option<f32> {
    const EDGE_INSET: f32 = 2.0;

    let min_y = plot.y + EDGE_INSET;
    let max_y = plot.y + plot.height - SAMPLE_LABEL_HEIGHT - EDGE_INSET;
    let preferred_below = point_y + point_radius + SAMPLE_LABEL_POINT_GAP;
    if preferred_below >= min_y && preferred_below <= max_y {
        return Some(preferred_below);
    }

    let preferred_above = point_y - point_radius - SAMPLE_LABEL_POINT_GAP - SAMPLE_LABEL_HEIGHT;
    (preferred_above >= min_y && preferred_above <= max_y).then_some(preferred_above)
}

#[cfg(test)]
#[path = "tests/points.rs"]
mod tests;

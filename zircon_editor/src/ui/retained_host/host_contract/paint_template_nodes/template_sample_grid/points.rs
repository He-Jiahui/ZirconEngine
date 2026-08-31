use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::paint_geometry::intersect;
use super::super::super::paint_text::measure_runtime_text_width;
use super::super::render_commands::HostPaintCommand;
use super::super::template_diamond_glyph::push_aa_diamond;
use super::geometry::SampleGridGeometry;
use super::metrics::{
    SampleGridMetrics, POINT_EDGE_INSET, POINT_INTERIOR_RADIUS, POINT_RADIUS, SAMPLE_LABEL_HEIGHT,
    SAMPLE_LABEL_MIN_WIDTH, SAMPLE_LABEL_POINT_GAP, TICK_FONT_SIZE, TICK_LINE_HEIGHT,
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
            POINT_RADIUS,
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
            POINT_INTERIOR_RADIUS,
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
            let Some(label_y) = selected_sample_label_y(y, &geometry.plot) else {
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

fn selected_sample_label_y(point_y: f32, plot: &FrameRect) -> Option<f32> {
    const EDGE_INSET: f32 = 2.0;

    let min_y = plot.y + EDGE_INSET;
    let max_y = plot.y + plot.height - SAMPLE_LABEL_HEIGHT - EDGE_INSET;
    let preferred_below = point_y + POINT_RADIUS as f32 + SAMPLE_LABEL_POINT_GAP;
    if preferred_below >= min_y && preferred_below <= max_y {
        return Some(preferred_below);
    }

    let preferred_above =
        point_y - POINT_RADIUS as f32 - SAMPLE_LABEL_POINT_GAP - SAMPLE_LABEL_HEIGHT;
    (preferred_above >= min_y && preferred_above <= max_y).then_some(preferred_above)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selected_sample_label_width_uses_runtime_text_measurement() {
        let label_width_limit = 480.0;
        let narrow_label = "iiiiiiiiiiiiiiii";
        let wide_label = "WWWWWWWWWWWWWWWW";
        let narrow_width = selected_sample_label_width(narrow_label, label_width_limit);
        let wide_width = selected_sample_label_width(wide_label, label_width_limit);

        assert_eq!(
            narrow_width,
            (measure_runtime_text_width(narrow_label, TICK_FONT_SIZE) + 12.0)
                .max(SAMPLE_LABEL_MIN_WIDTH)
                .min(label_width_limit * 0.6)
        );
        assert_eq!(
            wide_width,
            (measure_runtime_text_width(wide_label, TICK_FONT_SIZE) + 12.0)
                .max(SAMPLE_LABEL_MIN_WIDTH)
                .min(label_width_limit * 0.6)
        );
        assert!(
            wide_width > narrow_width,
            "selected labels need their actual glyph advances rather than a character count"
        );
    }

    #[test]
    fn selected_sample_label_collapses_when_the_plot_cannot_fit_its_minimum_width() {
        assert_eq!(
            selected_sample_label_width("Blend source", SAMPLE_LABEL_MIN_WIDTH / 0.6 - 0.1),
            0.0
        );
        assert_eq!(selected_sample_label_width("Blend source", f32::NAN), 0.0);
    }

    #[test]
    fn selected_sample_label_uses_the_unreal_below_key_placement_near_the_plot_top() {
        let plot = FrameRect {
            x: 20.0,
            y: 30.0,
            width: 240.0,
            height: 180.0,
        };

        let point_y = plot.y + POINT_EDGE_INSET;
        let label_y = selected_sample_label_y(point_y, &plot).unwrap();

        assert!(label_y > point_y);
        assert_eq!(label_y - (point_y + POINT_RADIUS as f32), 4.0);
        assert!(label_y + SAMPLE_LABEL_HEIGHT <= plot.y + plot.height);
    }

    #[test]
    fn selected_sample_label_prefers_below_and_flips_above_near_the_plot_bottom() {
        let plot = FrameRect {
            x: 20.0,
            y: 30.0,
            width: 240.0,
            height: 180.0,
        };

        let middle_label_y = selected_sample_label_y(plot.y + 90.0, &plot).unwrap();
        let bottom_point_y = plot.y + plot.height - POINT_EDGE_INSET;
        let bottom_label_y = selected_sample_label_y(bottom_point_y, &plot).unwrap();

        assert!(middle_label_y > plot.y + 90.0);
        assert!(bottom_label_y + SAMPLE_LABEL_HEIGHT < bottom_point_y);
        assert_eq!(middle_label_y - (plot.y + 90.0 + POINT_RADIUS as f32), 4.0);
        assert!(middle_label_y >= plot.y);
        assert!(bottom_label_y >= plot.y);
    }

    #[test]
    fn selected_sample_label_is_omitted_when_neither_side_can_fit() {
        let plot = FrameRect {
            x: 20.0,
            y: 30.0,
            width: 240.0,
            height: 32.0,
        };

        assert_eq!(
            selected_sample_label_y(plot.y + plot.height * 0.5, &plot),
            None
        );
    }

    #[test]
    fn selected_sample_label_centers_on_the_point_and_clamps_to_plot_edges() {
        let plot = FrameRect {
            x: 20.0,
            y: 30.0,
            width: 240.0,
            height: 180.0,
        };
        let label_width = 54.0;

        assert_eq!(selected_sample_label_x(140.0, label_width, &plot), 113.0);
        assert_eq!(selected_sample_label_x(21.0, label_width, &plot), 22.0);
        assert_eq!(selected_sample_label_x(259.0, label_width, &plot), 204.0);
    }
}

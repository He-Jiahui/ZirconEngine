use super::super::metrics::POINT_EDGE_INSET;
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
    let label_y = selected_sample_label_y(point_y, &plot, 5.0).unwrap();

    assert!(label_y > point_y);
    assert_eq!(label_y - (point_y + 5.0), 4.0);
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

    let middle_label_y = selected_sample_label_y(plot.y + 90.0, &plot, 5.0).unwrap();
    let bottom_point_y = plot.y + plot.height - POINT_EDGE_INSET;
    let bottom_label_y = selected_sample_label_y(bottom_point_y, &plot, 5.0).unwrap();

    assert!(middle_label_y > plot.y + 90.0);
    assert!(bottom_label_y + SAMPLE_LABEL_HEIGHT < bottom_point_y);
    assert_eq!(middle_label_y - (plot.y + 90.0 + 5.0), 4.0);
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
        selected_sample_label_y(plot.y + plot.height * 0.5, &plot, 5.0),
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

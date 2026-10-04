use super::*;

#[test]
fn collapsed_sample_grid_has_no_plot_area() {
    let geometry = SampleGridGeometry::from_frame(
        &FrameRect {
            x: 0.0,
            y: 0.0,
            width: 0.0,
            height: 160.0,
        },
        POINT_EDGE_INSET,
    );

    assert_eq!(geometry.plot.width, 0.0);
    assert_eq!(geometry.plot.height, 0.0);
}

#[test]
fn compact_plot_centers_points_instead_of_applying_an_oversized_inset() {
    let geometry = SampleGridGeometry {
        outer: FrameRect::default(),
        plot: FrameRect {
            x: 10.0,
            y: 20.0,
            width: 8.0,
            height: 10.0,
        },
        point_edge_inset: POINT_EDGE_INSET,
    };

    for value in [0.0, 0.5, 1.0] {
        assert_eq!(geometry.point_x_for_value(value, 0.0, 1.0), 14.0);
        assert_eq!(geometry.point_y_for_value(value, 0.0, 1.0), 25.0);
    }
}

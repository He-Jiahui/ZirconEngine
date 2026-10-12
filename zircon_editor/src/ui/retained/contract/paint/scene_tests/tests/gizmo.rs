use super::support::{paint_nodes, pixel_at, styled_node};

#[test]
fn viewport_gizmo_center_paints_native_axis_rod_and_facets() {
    let bytes = paint_nodes(
        96,
        96,
        vec![styled_node(
            "WorkbenchViewportGizmoCenter",
            36.0,
            42.0,
            36.0,
            31.0,
            [49, 93, 159, 255],
        )],
    );

    assert_ne!(pixel_at(&bytes, 96, 54, 24), [0, 0, 0, 255]);
    assert_ne!(pixel_at(&bytes, 96, 42, 46), pixel_at(&bytes, 96, 42, 68));
    assert_ne!(pixel_at(&bytes, 96, 68, 58), pixel_at(&bytes, 96, 42, 58));
}

#[test]
fn viewport_axis_line_uses_authored_background_color_before_axis_fallback() {
    let bytes = paint_nodes(
        96,
        48,
        vec![styled_node(
            "WorkbenchViewportAxisX",
            12.0,
            20.0,
            60.0,
            4.0,
            [10, 80, 190, 255],
        )],
    );

    let line = pixel_at(&bytes, 96, 34, 22);
    assert!(line[2] > line[0]);
}

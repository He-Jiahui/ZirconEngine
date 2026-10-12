use super::support::{luma, paint_nodes, pixel_at, styled_node};

#[test]
fn viewport_floor_grate_paints_repeating_native_slots() {
    let bytes = paint_nodes(
        80,
        80,
        vec![styled_node(
            "WorkbenchViewportFloorGrateRight",
            10.0,
            8.0,
            42.0,
            48.0,
            [0, 0, 0, 140],
        )],
    );

    assert_ne!(pixel_at(&bytes, 80, 16, 20), [0, 0, 0, 255]);
    assert_ne!(pixel_at(&bytes, 80, 20, 20), pixel_at(&bytes, 80, 16, 20));
}

#[test]
fn viewport_grid_line_paints_native_glow_band() {
    let bytes = paint_nodes(
        120,
        80,
        vec![styled_node(
            "WorkbenchViewportGridH2",
            20.0,
            40.0,
            80.0,
            1.0,
            [145, 155, 157, 87],
        )],
    );

    assert!(luma(pixel_at(&bytes, 120, 48, 40)) > luma(pixel_at(&bytes, 120, 48, 39)));
    assert_ne!(pixel_at(&bytes, 120, 48, 39), [0, 0, 0, 255]);
}

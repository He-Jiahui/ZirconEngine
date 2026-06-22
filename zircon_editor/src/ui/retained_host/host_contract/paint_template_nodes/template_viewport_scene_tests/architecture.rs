use super::support::{luma, paint_nodes, pixel_at, styled_node};

#[test]
fn viewport_back_door_paints_inset_panel_lines() {
    let bytes = paint_nodes(
        120,
        96,
        vec![styled_node(
            "WorkbenchViewportBackDoor",
            20.0,
            18.0,
            80.0,
            50.0,
            [75, 85, 89, 214],
        )],
    );

    assert!(luma(pixel_at(&bytes, 120, 32, 26)) > luma(pixel_at(&bytes, 120, 32, 32)));
    assert_ne!(pixel_at(&bytes, 120, 60, 42), pixel_at(&bytes, 120, 32, 42));
}

#[test]
fn viewport_wall_detail_paints_internal_line_grid() {
    let bytes = paint_nodes(
        140,
        120,
        vec![styled_node(
            "WorkbenchViewportWallDetailCenterLines",
            20.0,
            18.0,
            100.0,
            80.0,
            [170, 190, 199, 51],
        )],
    );

    assert!(luma(pixel_at(&bytes, 140, 48, 49)) > luma(pixel_at(&bytes, 140, 48, 24)));
    assert_ne!(pixel_at(&bytes, 140, 70, 28), pixel_at(&bytes, 140, 48, 24));
}

#[test]
fn viewport_side_stairs_paints_step_lines() {
    let bytes = paint_nodes(
        120,
        96,
        vec![styled_node(
            "WorkbenchViewportSideLeftStairs",
            20.0,
            18.0,
            90.0,
            60.0,
            [184, 194, 194, 56],
        )],
    );

    assert!(luma(pixel_at(&bytes, 120, 34, 28)) > luma(pixel_at(&bytes, 120, 34, 24)));
}

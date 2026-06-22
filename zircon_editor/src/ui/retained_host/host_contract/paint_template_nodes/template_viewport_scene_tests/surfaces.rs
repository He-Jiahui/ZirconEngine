use super::support::{luma, paint_nodes, paint_nodes_with_background, pixel_at, styled_node};

#[test]
fn viewport_ceiling_surface_paints_ribs_and_lower_shadow() {
    let bytes = paint_nodes(
        120,
        80,
        vec![styled_node(
            "WorkbenchViewportCeiling",
            10.0,
            10.0,
            100.0,
            40.0,
            [21, 26, 29, 255],
        )],
    );

    assert_ne!(pixel_at(&bytes, 120, 52, 20), pixel_at(&bytes, 120, 50, 20));
    assert!(luma(pixel_at(&bytes, 120, 60, 48)) < luma(pixel_at(&bytes, 120, 60, 30)));
}

#[test]
fn viewport_back_wall_surface_paints_panel_grid() {
    let bytes = paint_nodes(
        140,
        120,
        vec![styled_node(
            "WorkbenchViewportBackWall",
            20.0,
            18.0,
            100.0,
            80.0,
            [27, 29, 31, 255],
        )],
    );

    assert_ne!(pixel_at(&bytes, 140, 70, 50), pixel_at(&bytes, 140, 58, 50));
    assert_ne!(pixel_at(&bytes, 140, 42, 45), pixel_at(&bytes, 140, 42, 44));
}

#[test]
fn viewport_floor_surface_paints_depth_lines_and_bottom_shadow() {
    let bytes = paint_nodes(
        140,
        100,
        vec![styled_node(
            "WorkbenchViewportFloor",
            10.0,
            10.0,
            110.0,
            64.0,
            [35, 38, 37, 255],
        )],
    );

    assert_ne!(pixel_at(&bytes, 140, 58, 46), pixel_at(&bytes, 140, 58, 45));
    assert!(luma(pixel_at(&bytes, 140, 60, 70)) < luma(pixel_at(&bytes, 140, 60, 34)));
}

#[test]
fn viewport_layout_containers_do_not_paint_fallback_surface() {
    let bytes = paint_nodes_with_background(
        90,
        72,
        [9, 11, 13, 255],
        vec![
            styled_node(
                "WorkbenchViewportSurface",
                8.0,
                8.0,
                72.0,
                48.0,
                [0, 0, 0, 0],
            ),
            styled_node(
                "WorkbenchViewportGizmoPanel",
                18.0,
                18.0,
                32.0,
                24.0,
                [0, 0, 0, 0],
            ),
        ],
    );

    assert_eq!(pixel_at(&bytes, 90, 20, 20), [9, 11, 13, 255]);
    assert_eq!(pixel_at(&bytes, 90, 70, 50), [9, 11, 13, 255]);
}

use super::support::{luma, paint_nodes, pixel_at, styled_node};

#[test]
fn viewport_handrail_paints_posts_beyond_authored_rail_rect() {
    let bytes = paint_nodes(
        140,
        80,
        vec![styled_node(
            "WorkbenchViewportHandrailLeft",
            10.0,
            10.0,
            100.0,
            4.0,
            [179, 113, 48, 122],
        )],
    );

    assert_ne!(pixel_at(&bytes, 140, 47, 36), [0, 0, 0, 255]);
    assert_eq!(pixel_at(&bytes, 140, 20, 36), [0, 0, 0, 255]);
}

#[test]
fn viewport_cargo_inner_paints_frame_without_cargo_body_fill() {
    let bytes = paint_nodes(
        120,
        96,
        vec![styled_node(
            "WorkbenchViewportCargoRightInner",
            20.0,
            18.0,
            80.0,
            44.0,
            [0, 0, 0, 0],
        )],
    );

    assert_ne!(pixel_at(&bytes, 120, 20, 18), [0, 0, 0, 255]);
    assert_ne!(pixel_at(&bytes, 120, 47, 28), [0, 0, 0, 255]);
    assert_eq!(pixel_at(&bytes, 120, 58, 30), [0, 0, 0, 255]);
}

#[test]
fn viewport_selected_prop_paints_box_facets_instead_of_cargo_slots() {
    let bytes = paint_nodes(
        120,
        96,
        vec![
            styled_node(
                "WorkbenchViewportPropTop",
                24.0,
                18.0,
                64.0,
                16.0,
                [42, 48, 53, 255],
            ),
            styled_node(
                "WorkbenchViewportPropBody",
                24.0,
                34.0,
                64.0,
                44.0,
                [32, 37, 40, 255],
            ),
        ],
    );

    assert!(luma(pixel_at(&bytes, 120, 44, 37)) > luma(pixel_at(&bytes, 120, 44, 50)));
    assert!(luma(pixel_at(&bytes, 120, 85, 50)) < luma(pixel_at(&bytes, 120, 44, 50)));
    assert!(luma(pixel_at(&bytes, 120, 44, 21)) > luma(pixel_at(&bytes, 120, 44, 30)));
}

use super::support::{luma, paint_nodes, paint_nodes_with_background, pixel_at, styled_node};

#[test]
fn viewport_soft_light_paints_layered_center_intensity() {
    let bytes = paint_nodes(
        140,
        120,
        vec![styled_node(
            "WorkbenchViewportLightwashCenter",
            20.0,
            20.0,
            90.0,
            60.0,
            [174, 198, 211, 96],
        )],
    );

    assert!(luma(pixel_at(&bytes, 140, 65, 50)) > luma(pixel_at(&bytes, 140, 22, 22)));
}

#[test]
fn viewport_soft_shadow_darkens_toward_center() {
    let bytes = paint_nodes_with_background(
        140,
        120,
        [48, 54, 58, 255],
        vec![styled_node(
            "WorkbenchViewportShadowTopBay",
            20.0,
            20.0,
            90.0,
            60.0,
            [0, 0, 0, 128],
        )],
    );

    let center = pixel_at(&bytes, 140, 65, 50);
    assert!(luma(center) < luma([48, 54, 58, 255]));
    assert!(luma(center) < luma(pixel_at(&bytes, 140, 22, 22)));
}

#[test]
fn viewport_wall_light_paints_hot_core_over_soft_strip() {
    let bytes = paint_nodes(
        120,
        72,
        vec![styled_node(
            "WorkbenchViewportWallLightFarRight",
            30.0,
            30.0,
            56.0,
            8.0,
            [217, 230, 233, 144],
        )],
    );

    assert!(luma(pixel_at(&bytes, 120, 58, 31)) > luma(pixel_at(&bytes, 120, 58, 36)));
}

#[test]
fn viewport_beacon_paints_hot_inner_strip() {
    let bytes = paint_nodes(
        80,
        96,
        vec![styled_node(
            "WorkbenchViewportWallBeaconLeft",
            36.0,
            18.0,
            8.0,
            56.0,
            [225, 148, 80, 168],
        )],
    );

    assert!(luma(pixel_at(&bytes, 80, 39, 46)) > luma(pixel_at(&bytes, 80, 36, 46)));
}

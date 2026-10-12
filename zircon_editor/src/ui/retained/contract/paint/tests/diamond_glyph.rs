use super::*;

#[test]
fn diamond_raster_contains_transparent_opaque_and_fractional_pixels() {
    let rgba = diamond_pixels(7, [20, 30, 40, 255]);
    let mut alphas = rgba.chunks_exact(4).map(|pixel| pixel[3]);

    assert!(alphas.clone().any(|alpha| alpha == 0));
    assert!(alphas.clone().any(|alpha| alpha == 255));
    assert!(alphas.any(|alpha| (1..=254).contains(&alpha)));
}

#[test]
fn repeated_diamond_rasters_share_pixel_storage() {
    let key = DiamondRasterKey {
        source_edge: 7,
        target_edge_bits: 7.0_f32.to_bits(),
        color: [50, 60, 70, 255],
    };
    let first = cached_diamond_raster(key);
    let second = cached_diamond_raster(key);

    assert_eq!(first.resource_key, second.resource_key);
    assert!(Arc::ptr_eq(&first.rgba, &second.rgba));
}

#[test]
fn one_diamond_emits_one_image_command() {
    let mut commands = Vec::new();
    push_aa_diamond(
        &mut commands,
        20.0,
        30.0,
        3.0,
        [80, 90, 100, 255],
        &FrameRect {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 100.0,
        },
        7,
        1.0,
    );

    assert_eq!(commands.len(), 1);
    assert_eq!(commands[0].frame.x, 16.5);
    assert_eq!(commands[0].frame.y, 26.5);
    assert_eq!(commands[0].frame.width, 7.0);
    assert_eq!(commands[0].frame.height, 7.0);
    assert!(commands[0].image_pixels.is_some());
}

#[test]
fn fractional_diamond_keeps_physical_target_extent_and_ceil_source_resolution() {
    let mut commands = Vec::new();
    push_aa_diamond(
        &mut commands,
        20.0,
        30.0,
        3.25,
        [80, 90, 100, 255],
        &FrameRect {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 100.0,
        },
        7,
        1.0,
    );

    assert_eq!(commands.len(), 1);
    assert_eq!(commands[0].frame.x, 16.25);
    assert_eq!(commands[0].frame.y, 26.25);
    assert_eq!(commands[0].frame.width, 7.5);
    assert_eq!(commands[0].frame.height, 7.5);
    let image = commands[0]
        .image_pixels
        .as_ref()
        .expect("fractional diamond should use the cached local raster");
    assert_eq!(image.width, 8);
    assert_eq!(image.height, 8);
    let target_edge_bits = format!("{:08x}", 7.5_f32.to_bits());
    assert!(image.resource_key.contains(target_edge_bits.as_str()));
}

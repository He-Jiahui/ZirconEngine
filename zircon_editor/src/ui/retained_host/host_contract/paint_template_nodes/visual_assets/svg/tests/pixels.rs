use std::sync::Arc;

use super::super::super::RasterTargetSize;
use super::super::parse::parse_svg_tree_data;
use super::{downsample_rgba, render_svg_tree_pixels};

#[test]
fn two_x_resolve_averages_opaque_color_samples_in_linear_light() {
    let source = [
        255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 255, 255, 255, 255,
    ];

    assert_eq!(downsample_rgba(&source, 2, 1, 1, 2), [188, 188, 188, 255]);
}

#[test]
fn two_x_resolve_ignores_rgb_hidden_by_zero_alpha() {
    let source = [255, 0, 0, 0, 0, 0, 255, 255, 255, 0, 0, 0, 255, 0, 0, 0];

    assert_eq!(downsample_rgba(&source, 2, 1, 1, 2), [0, 0, 255, 64]);
}

#[test]
fn four_x_resolve_uses_all_sixteen_coverage_samples() {
    let mut source = vec![0_u8; 4 * 4 * 4];
    source[0..4].copy_from_slice(&[255, 255, 255, 255]);

    assert_eq!(downsample_rgba(&source, 4, 1, 1, 4), [255, 255, 255, 16]);
}

#[test]
fn vector_raster_resolves_fractional_edges_at_the_physical_target() {
    let tree = parse_svg_tree_data(
        br##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 10 10">
                    <circle cx="5" cy="5" r="3.4" fill="#ffffff"/>
                </svg>"##,
        None,
    )
    .expect("parse vector fixture");
    let target = RasterTargetSize::new(9, 9).expect("physical target");

    let image = render_svg_tree_pixels(Arc::new(tree), target, None)
        .expect("render supersampled vector fixture");
    let alpha = image.rgba.chunks_exact(4).map(|pixel| pixel[3]);

    assert_eq!((image.width, image.height), (9, 9));
    assert!(alpha.clone().any(|value| value == 0));
    assert!(alpha.clone().any(|value| value == 255));
    assert!(alpha.into_iter().any(|value| (1..=254).contains(&value)));
}

#[test]
fn vector_raster_centers_source_aspect_inside_the_full_requested_target() {
    let tree = parse_svg_tree_data(
        br##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 10">
                    <rect width="20" height="10" fill="#ffffff"/>
                </svg>"##,
        None,
    )
    .expect("parse vector fixture");
    let target = RasterTargetSize::new(100, 100).expect("physical target");

    let image = render_svg_tree_pixels(Arc::new(tree), target, None)
        .expect("render aspect-preserving vector fixture");

    assert_eq!((image.width, image.height), (100, 100));
    let visible_rows = image
        .rgba
        .chunks_exact(100 * 4)
        .enumerate()
        .filter_map(|(row, pixels)| {
            pixels
                .chunks_exact(4)
                .any(|pixel| pixel[3] > 0)
                .then_some(row)
        })
        .collect::<Vec<_>>();
    assert_eq!(visible_rows.first(), Some(&25));
    assert_eq!(visible_rows.last(), Some(&74));
}

use std::collections::BTreeSet;

use super::{RasterTargetSize, MAX_VECTOR_RASTER_EDGE};

#[test]
fn small_icon_vector_source_uses_four_x_local_supersampling() {
    let target = RasterTargetSize::new(20, 18).expect("display target");
    let (source, scale) = target.vector_supersampled_source();

    assert_eq!(scale, 4);
    assert_eq!((source.width, source.height), (80, 72));
}

#[test]
fn larger_vector_source_uses_two_x_local_supersampling() {
    let target = RasterTargetSize::new(64, 48).expect("display target");
    let (source, scale) = target.vector_supersampled_source();

    assert_eq!(scale, 2);
    assert_eq!((source.width, source.height), (128, 96));
}

#[test]
fn vector_source_never_exceeds_the_raster_edge_limit() {
    let target = RasterTargetSize::new(MAX_VECTOR_RASTER_EDGE, 32).expect("large target");
    let (source, scale) = target.vector_supersampled_source();

    assert_eq!(scale, 1);
    assert_eq!(source.width, MAX_VECTOR_RASTER_EDGE);
    assert_eq!(source.height, 32);
}

#[test]
fn raster_target_fits_non_square_sources_without_distortion() {
    let target = RasterTargetSize::new(100, 100).expect("display target");

    let wide = target
        .fit_preserving_aspect(200.0, 100.0)
        .expect("wide source");
    let tall = target
        .fit_preserving_aspect(100.0, 200.0)
        .expect("tall source");

    assert_eq!((wide.width, wide.height), (100, 50));
    assert_eq!((tall.width, tall.height), (50, 100));
}

#[test]
fn preview_raster_targets_quantize_up_without_exceeding_the_edge_limit() {
    let target = RasterTargetSize::new(121, 125).expect("physical target");
    let bounded =
        RasterTargetSize::new(MAX_VECTOR_RASTER_EDGE - 1, 1).expect("bounded physical target");

    assert_eq!(
        target.quantized_up(8),
        RasterTargetSize::new(128, 128).unwrap()
    );
    assert_eq!(
        bounded.quantized_up(8),
        RasterTargetSize::new(MAX_VECTOR_RASTER_EDGE, 8).unwrap()
    );
    assert_eq!(target.quantized_up(1), target);
}

#[test]
fn continuous_resize_uses_bounded_vector_cache_buckets() {
    let exact_sizes = (1..=512)
        .map(|edge| RasterTargetSize::new(edge, edge).unwrap())
        .collect::<BTreeSet<_>>();
    let bucketed_sizes = exact_sizes
        .iter()
        .copied()
        .map(RasterTargetSize::vector_cache_bucket)
        .collect::<BTreeSet<_>>();

    assert_eq!(exact_sizes.len(), 512);
    assert_eq!(bucketed_sizes.len(), 80);
    assert_eq!(
        RasterTargetSize::new(17, 19).unwrap().vector_cache_bucket(),
        RasterTargetSize::new(17, 19).unwrap()
    );
    assert_eq!(
        RasterTargetSize::new(121, 125)
            .unwrap()
            .vector_cache_bucket(),
        RasterTargetSize::new(121, 125).unwrap()
    );
}

#[test]
fn non_square_vector_targets_keep_their_physical_aspect_ratio() {
    let target = RasterTargetSize::new(41, 43).expect("non-square physical target");

    assert_eq!(target.vector_cache_bucket(), target);
}

#[test]
fn frame_target_ceil_preserves_fractional_physical_pixel_coverage() {
    let at_125_percent =
        RasterTargetSize::from_frame(17.0 * 1.25, 13.0 * 1.25).expect("125% target");
    let at_150_percent = RasterTargetSize::from_frame(17.0 * 1.5, 13.0 * 1.5).expect("150% target");

    assert_eq!((at_125_percent.width, at_125_percent.height), (22, 17));
    assert_eq!((at_150_percent.width, at_150_percent.height), (26, 20));
}

#[test]
fn frame_target_clamps_after_rounding_up_to_the_physical_edge_limit() {
    let target = RasterTargetSize::from_frame(
        MAX_VECTOR_RASTER_EDGE as f32 + 0.25,
        MAX_VECTOR_RASTER_EDGE as f32 + 128.0,
    )
    .expect("bounded target");

    assert_eq!(target.width, MAX_VECTOR_RASTER_EDGE);
    assert_eq!(target.height, MAX_VECTOR_RASTER_EDGE);
}

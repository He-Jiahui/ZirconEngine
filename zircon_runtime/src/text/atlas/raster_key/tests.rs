use super::*;
use std::hint::black_box;
use std::time::Instant;

#[test]
fn text_atlas_key_rebuckets_on_scale_change() {
    let base = request(12.0, 1.0, 0.0);
    let high_dpi = request(12.0, 2.0, 0.0);

    let base_key = GlyphRasterKey::from_request(base);
    let high_dpi_key = GlyphRasterKey::from_request(high_dpi);

    assert_eq!(base_key.px_size_bucket, 12);
    assert_eq!(high_dpi_key.px_size_bucket, 24);
    assert_ne!(base_key, high_dpi_key);
}

#[test]
fn text_raster_subpixel_bins_are_part_of_bitmap_key() {
    let left = GlyphRasterKey::from_request(request(13.0, 1.0, 0.1));
    let middle = GlyphRasterKey::from_request(request(13.0, 1.0, 0.45));
    let right = GlyphRasterKey::from_request(request(13.0, 1.0, 0.8));

    assert_eq!(left.subpixel_bin, 0);
    assert_eq!(middle.subpixel_bin, 1);
    assert_eq!(right.subpixel_bin, 2);
    assert_ne!(left, middle);
    assert_ne!(middle, right);
}

#[test]
fn optimization_batch_20260831fa_runtime566_signed_fraction_preserves_subpixel_bins() {
    for screen_x in [
        -4096.75_f32,
        -4.0,
        -1.0,
        -0.9999,
        -0.6667,
        -0.3334,
        -0.0,
        0.0,
        0.3333,
        0.6666,
        0.9999,
        1.0,
        4096.75,
    ] {
        let legacy = (screen_x.rem_euclid(1.0) * SUBPIXEL_BIN_COUNT as f32).floor() as u8;
        assert_eq!(subpixel_bin_for_screen_x(screen_x), legacy);
    }
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260831fa_runtime566_subpixel_fraction_p95() {
    const SAMPLE_PAIRS: usize = 13;
    const ITERATIONS: u64 = 20_000_000;
    let mut legacy = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy.push(measure_fraction(false, ITERATIONS));
            optimized.push(measure_fraction(true, ITERATIONS));
        } else {
            optimized.push(measure_fraction(true, ITERATIONS));
            legacy.push(measure_fraction(false, ITERATIONS));
        }
    }
    let legacy_p95_ns = percentile(&legacy, 95);
    let optimized_p95_ns = percentile(&optimized, 95);
    println!(
        "RUNTIME566_SUBPIXEL_FRACTION_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
iterations={ITERATIONS} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        csv(&legacy),
        csv(&optimized)
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(85));
}

fn measure_fraction(optimized: bool, iterations: u64) -> u128 {
    let started = Instant::now();
    let mut checksum = 0_u64;
    for index in 0..iterations {
        let whole = (index & 4095) as f32;
        let fraction = ((index >> 12) & 31) as f32 * (1.0 / 32.0);
        let screen_x = black_box(if index & 127 == 0 {
            -whole - fraction
        } else {
            whole + fraction
        });
        checksum += u64::from(if optimized {
            subpixel_bin_for_screen_x(screen_x)
        } else {
            (screen_x.rem_euclid(1.0) * SUBPIXEL_BIN_COUNT as f32).floor() as u8
        });
    }
    black_box(checksum);
    started.elapsed().as_nanos().max(1)
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    sorted[(sorted.len() * percentile).div_ceil(100).saturating_sub(1)]
}

fn csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}

#[test]
fn text_raster_vertical_subpixel_phase_is_part_of_bitmap_key() {
    let base = GlyphRasterKey::from_request(request(13.0, 1.0, 0.0));
    let shifted = GlyphRasterKey {
        vertical_subpixel_bin: 2,
        ..base
    };

    assert_ne!(base, shifted);
}

#[test]
fn text_raster_subpixel_placement_snaps_quad_to_bin_origin() {
    let placement = GlyphRasterPlacement::from_request(request(13.0, 1.0, 10.45));

    assert_eq!(placement.subpixel_bin, 1);
    assert_near(placement.requested_x, 10.45);
    assert_near(placement.snapped_x, 10.0 + 1.0 / 3.0);
}

#[test]
fn text_raster_subpixel_placement_uses_last_bin_below_next_pixel() {
    let placement = GlyphRasterPlacement::from_request(request(13.0, 1.0, 10.99));

    assert_eq!(placement.subpixel_bin, 2);
    assert_near(placement.snapped_x, 10.0 + 2.0 / 3.0);
}

#[test]
fn text_raster_pixel_snap_disables_subpixel_bins() {
    let mut snapped = request(13.0, 1.0, 0.8);
    snapped.snap_to_pixel = true;

    let key = GlyphRasterKey::from_request(snapped);
    let placement = GlyphRasterPlacement::from_request(snapped);

    assert_eq!(key.subpixel_bin, 0);
    assert_eq!(placement.subpixel_bin, 0);
    assert_near(placement.snapped_x, 1.0);
}

#[test]
fn text_raster_grayscale_placement_preserves_fractional_x() {
    let mut request = request(13.0, 1.0, 10.45);
    request.smoothing = GlyphSmoothingMode::Grayscale;

    let placement = GlyphRasterPlacement::from_request(request);

    assert_eq!(placement.subpixel_bin, 0);
    assert_near(placement.snapped_x, 10.45);
}

#[test]
fn text_sdf_raster_key_normalizes_hinting_and_smoothing() {
    let mut request = request(32.0, 1.5, 0.45);
    request.format = GlyphAtlasFormat::Sdf;
    request.hinting = GlyphHintingMode::Full;
    request.smoothing = GlyphSmoothingMode::Subpixel;

    let key = GlyphRasterKey::from_request(request);

    assert_eq!(key.px_size_bucket, 48);
    assert_eq!(key.subpixel_bin, 0);
    assert_eq!(key.hinting, GlyphHintingMode::None);
    assert_eq!(key.smoothing, GlyphSmoothingMode::None);
}

#[test]
fn text_sdf_raster_placement_preserves_fractional_x() {
    let mut request = request(32.0, 1.5, 7.42);
    request.format = GlyphAtlasFormat::Sdf;

    let placement = GlyphRasterPlacement::from_request(request);

    assert_eq!(placement.subpixel_bin, 0);
    assert_near(placement.snapped_x, 7.42);
}

#[test]
fn text_subpixel_mask_raster_key_keeps_subpixel_bins_in_rgba_atlas() {
    let mut request = request(13.0, 1.0, 10.45);
    request.format = GlyphAtlasFormat::SubpixelMask;

    let key = GlyphRasterKey::from_request(request);
    let placement = GlyphRasterPlacement::from_request(request);

    assert_eq!(key.format, GlyphAtlasFormat::SubpixelMask);
    assert_eq!(key.subpixel_bin, 1);
    assert_eq!(placement.subpixel_bin, 1);
    assert_near(placement.snapped_x, 10.0 + 1.0 / 3.0);
}

#[test]
fn text_raster_synthetic_style_changes_key_identity() {
    let plain = GlyphRasterKey::from_request(request(14.0, 1.0, 0.0));
    let mut synthetic_request = request(14.0, 1.0, 0.0);
    synthetic_request.synthetic = SyntheticGlyphStyle {
        bold: true,
        oblique: true,
    };

    let synthetic = GlyphRasterKey::from_request(synthetic_request);

    assert_ne!(plain, synthetic);
    assert!(synthetic.synthetic.bold);
    assert!(synthetic.synthetic.oblique);
}

fn request(logical_px: f32, scale_factor: f32, screen_x: f32) -> GlyphRasterRequest {
    GlyphRasterRequest {
        face: InstancedFaceId(7),
        glyph_id: 42,
        logical_px,
        scale_factor,
        screen_x,
        snap_to_pixel: false,
        format: GlyphAtlasFormat::AlphaMask,
        hinting: GlyphHintingMode::Vertical,
        smoothing: GlyphSmoothingMode::Subpixel,
        synthetic: SyntheticGlyphStyle::default(),
    }
}

fn assert_near(actual: f32, expected: f32) {
    assert!(
        (actual - expected).abs() <= 0.001,
        "expected {actual} to be near {expected}"
    );
}

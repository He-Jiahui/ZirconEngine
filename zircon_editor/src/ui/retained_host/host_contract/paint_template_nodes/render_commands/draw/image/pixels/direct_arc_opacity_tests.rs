use std::hint::black_box;
use std::sync::Arc;
use std::time::Instant;

use super::*;

const PIXEL_COUNT: usize = 16 * 1024;
const OPERATIONS_PER_SAMPLE: usize = 128;
const SAMPLE_PAIRS: usize = 21;

#[test]
fn optimization_batch_20260826hm_editor205_preserves_rgba_and_scales_alpha() {
    let source = Arc::<[u8]>::from([10_u8, 20, 30, 200, 40, 50, 60, 255]);
    let pixels = pixels_with_opacity(&source, 0.5);

    assert_eq!(pixels.as_ref(), &[10, 20, 30, 100, 40, 50, 60, 128]);
    assert_eq!(source.as_ref(), &[10, 20, 30, 200, 40, 50, 60, 255]);
    assert_eq!(Arc::strong_count(&pixels), 1);

    let opaque = pixels_with_opacity(&source, 1.0);
    assert!(Arc::ptr_eq(&opaque, &source));
}

#[test]
fn optimization_batch_20260826hm_editor205_copies_directly_into_arc_storage() {
    let source = include_str!("../pixels.rs");

    assert!(source.contains("return Arc::clone(source)"));
    assert!(source.contains("Arc::<[u8]>::from(source)"));
    assert!(source.contains("Arc::get_mut(&mut rgba)"));
    assert!(!source.contains("image.rgba.as_ref().to_vec()"));
    assert!(!source.contains("rgba.into()"));
}

#[test]
#[ignore = "managed release performance evidence"]
fn optimization_batch_20260826hm_editor205_direct_arc_image_opacity_release_benchmark() {
    let source = Arc::<[u8]>::from(
        (0..PIXEL_COUNT)
            .flat_map(|index| [31_u8, 79, 127, (index % 256) as u8])
            .collect::<Vec<_>>(),
    );

    let mut legacy_ns = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_ns = Vec::with_capacity(SAMPLE_PAIRS);
    for sample_index in 0..SAMPLE_PAIRS {
        let mut measure_legacy = || {
            let started = Instant::now();
            for _ in 0..OPERATIONS_PER_SAMPLE {
                black_box(legacy_pixels_with_opacity(black_box(&source), 1.0));
            }
            legacy_ns.push(started.elapsed().as_nanos().max(1));
        };
        let mut measure_optimized = || {
            let started = Instant::now();
            for _ in 0..OPERATIONS_PER_SAMPLE {
                black_box(pixels_with_opacity(black_box(&source), 1.0));
            }
            optimized_ns.push(started.elapsed().as_nanos().max(1));
        };
        if sample_index % 2 == 0 {
            measure_legacy();
            measure_optimized();
        } else {
            measure_optimized();
            measure_legacy();
        }
    }

    let legacy_p50_ns = percentile(&legacy_ns, 50);
    let legacy_p95_ns = percentile(&legacy_ns, 95);
    let optimized_p50_ns = percentile(&optimized_ns, 50);
    let optimized_p95_ns = percentile(&optimized_ns, 95);
    println!(
        "EDITOR205_DIRECT_ARC_IMAGE_OPACITY_BENCH_V1 \
         pixel_count={PIXEL_COUNT} operations_per_sample={OPERATIONS_PER_SAMPLE} \
         sample_pairs={SAMPLE_PAIRS} legacy_p50_ns={legacy_p50_ns} \
         legacy_p95_ns={legacy_p95_ns} optimized_p50_ns={optimized_p50_ns} \
         optimized_p95_ns={optimized_p95_ns} legacy_ns={} optimized_ns={}",
        samples(&legacy_ns),
        samples(&optimized_ns),
    );
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70),
        "optimized P95 {optimized_p95_ns}ns must be at most 70% of legacy P95 {legacy_p95_ns}ns"
    );
}

fn legacy_pixels_with_opacity(source: &Arc<[u8]>, opacity: f32) -> Arc<[u8]> {
    let mut rgba = source.as_ref().to_vec();
    for pixel in rgba.chunks_exact_mut(4) {
        pixel[3] = ((pixel[3] as f32 * opacity).round()).clamp(0.0, 255.0) as u8;
    }
    rgba.into()
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut ordered = samples.to_vec();
    ordered.sort_unstable();
    let rank = ordered.len().saturating_mul(percentile).div_ceil(100);
    ordered[rank.saturating_sub(1)]
}

fn samples(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}

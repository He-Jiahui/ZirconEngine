use std::hint::black_box;
use std::time::Instant;

use super::*;

const SAMPLE_PAIRS: usize = 17;
const KEYS_PER_SAMPLE: usize = 262_144;

#[test]
fn optimization_batch_fd_editor392_rgba_resource_key_preserves_bytes() {
    for (width, height, rgba) in [
        (0, 0, &[][..]),
        (1, 1, &[255, 0, 128, 255][..]),
        (1920, 1080, &[17; 16][..]),
        (u32::MAX, u32::MAX, &[3; 64][..]),
    ] {
        assert_eq!(
            rgba_resource_key(width, height, rgba),
            legacy_rgba_resource_key(width, height, rgba)
        );
    }

    let production = include_str!("../recording.rs")
        .split("#[cfg(test)]")
        .next()
        .expect("production source");
    assert!(!production.contains("format!("));
}

#[test]
#[ignore = "release performance gate"]
fn optimization_batch_fd_editor392_direct_rgba_resource_key_benchmark() {
    for _ in 0..4 {
        black_box(measure_keys(legacy_rgba_resource_key));
        black_box(measure_keys(rgba_resource_key));
    }

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair_index in 0..SAMPLE_PAIRS {
        if pair_index % 2 == 0 {
            legacy_samples.push(measure_keys(legacy_rgba_resource_key));
            optimized_samples.push(measure_keys(rgba_resource_key));
        } else {
            optimized_samples.push(measure_keys(rgba_resource_key));
            legacy_samples.push(measure_keys(legacy_rgba_resource_key));
        }
    }

    report_performance(&legacy_samples, &optimized_samples);
}

fn legacy_rgba_resource_key(image_width: u32, image_height: u32, rgba: &[u8]) -> String {
    let mut hasher = DefaultHasher::new();
    image_width.hash(&mut hasher);
    image_height.hash(&mut hasher);
    rgba.hash(&mut hasher);
    format!("rgba:{image_width}x{image_height}:{:016x}", hasher.finish())
}

fn measure_keys(mut build: impl FnMut(u32, u32, &[u8]) -> String) -> u128 {
    const PIXEL: &[u8] = &[255, 63, 127, 255];
    let started = Instant::now();
    let mut checksum = 0_usize;
    for index in 0..KEYS_PER_SAMPLE {
        let width = black_box(1 + index as u32 % 4_096);
        let height = black_box(1 + index as u32 % 2_160);
        let key = black_box(build(width, height, black_box(PIXEL)));
        checksum = checksum.wrapping_add(key.len());
        black_box(key);
    }
    black_box(checksum);
    started.elapsed().as_nanos().max(1)
}

fn report_performance(legacy_samples: &[u128], optimized_samples: &[u128]) {
    let legacy_p95 = nearest_rank_p95(legacy_samples);
    let optimized_p95 = nearest_rank_p95(optimized_samples);
    let improvement_percent =
        legacy_p95.saturating_sub(optimized_p95).saturating_mul(100) / legacy_p95.max(1);
    println!(
        "EDITOR392_DIRECT_RGBA_RESOURCE_KEY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} keys_per_sample={KEYS_PER_SAMPLE} legacy_ns={} optimized_ns={} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} improvement_percent={improvement_percent} threshold_percent=25",
        csv(legacy_samples),
        csv(optimized_samples),
    );
    assert!(
        optimized_p95 <= legacy_p95.saturating_mul(75) / 100,
        "direct RGBA resource keys must reduce P95 by at least 25%"
    );
}

fn nearest_rank_p95(samples: &[u128]) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * 95).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}

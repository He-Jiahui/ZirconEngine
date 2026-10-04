use std::hint::black_box;
use std::time::Instant;

use super::{source_axis_sample, source_sample_coordinate};

#[test]
fn optimization_batch_20260830et_editor554_reuses_x_axis_samples_across_rows() {
    let production = include_str!("../scaled.rs")
        .split("#[cfg(test)]")
        .next()
        .expect("scaled image production source");

    assert!(production.contains("let source_x_samples"));
    assert!(production.contains("for sample in &source_x_samples"));
    assert!(!production.contains("source_sample_coordinate(x,"));
}

#[test]
fn optimization_batch_20260830et_editor554_cached_samples_match_direct_coordinates() {
    for destination in 3..197 {
        let coordinate = source_sample_coordinate(destination, 2.25, 211.5, 127);
        let sample = source_axis_sample(destination, 2.25, 211.5, 127);
        let lower = coordinate.floor() as u32;

        assert_eq!(sample.lower, lower);
        assert_eq!(sample.upper, lower.saturating_add(1).min(126));
        assert_eq!(sample.mix, coordinate - lower as f32);
    }
}

#[test]
#[ignore = "deterministic performance marker"]
fn optimization_batch_20260830et_editor554_x_axis_sample_cache_benchmark() {
    const WIDTH: u32 = 512;
    const HEIGHT: u32 = 512;
    const SAMPLES: usize = 9;
    let mut legacy_samples = Vec::with_capacity(SAMPLES);
    let mut optimized_samples = Vec::with_capacity(SAMPLES);

    for _ in 0..SAMPLES {
        let started = Instant::now();
        let mut checksum = 0_u32;
        for _ in 0..HEIGHT {
            for x in 0..WIDTH {
                checksum ^= source_axis_sample(x, 0.25, 511.5, 384).lower;
            }
        }
        black_box(checksum);
        legacy_samples.push(started.elapsed());

        let started = Instant::now();
        let cached = (0..WIDTH)
            .map(|x| source_axis_sample(x, 0.25, 511.5, 384))
            .collect::<Vec<_>>();
        let mut checksum = 0_u32;
        for _ in 0..HEIGHT {
            for sample in &cached {
                checksum ^= sample.lower;
            }
        }
        black_box(checksum);
        optimized_samples.push(started.elapsed());
    }

    legacy_samples.sort_unstable();
    optimized_samples.sort_unstable();
    println!(
        "EDITOR554_X_AXIS_SAMPLE_CACHE_BENCH_V1 legacy={:?} optimized={:?}",
        legacy_samples[SAMPLES / 2],
        optimized_samples[SAMPLES / 2]
    );
}

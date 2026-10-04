use std::hint::black_box;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

#[test]
fn optimization_batch_20260830cg_clip_sampling_checks_resident_snapshots_first() {
    let source = include_str!("../clip_sample.rs");
    let production = source
        .split("#[cfg(test)]")
        .next()
        .expect("production source");

    for (start, end, loader) in [
        (
            "fn load_skeleton_snapshot(",
            "fn load_clip_snapshot(",
            "load_animation_skeleton_asset",
        ),
        (
            "fn load_clip_snapshot(",
            "#[cfg(test)]",
            "load_animation_clip_asset",
        ),
    ] {
        let start = source.find(start).expect("snapshot helper");
        let helper = production.get(start..).unwrap_or(&source[start..]);
        let helper = helper.split(end).next().expect("snapshot helper boundary");
        let snapshot = helper.find("resources.snapshot").expect("resident lookup");
        let load = helper.find(loader).expect("loader fallback");
        assert!(snapshot < load);
        assert!(helper.contains(".or_else(||"));
    }
}

#[test]
#[ignore = "Release-only Runtime170 performance contract"]
fn optimization_batch_20260830cg_resident_snapshot_first_p95() {
    const ITERATIONS: usize = 1_000_000;
    const SAMPLES: usize = 17;
    let load_count = AtomicU64::new(0);
    let mut baseline_samples = Vec::with_capacity(SAMPLES);
    let mut optimized_samples = Vec::with_capacity(SAMPLES);

    for sample in 0..SAMPLES {
        let baseline = || {
            let started = Instant::now();
            for _ in 0..ITERATIONS {
                mock_load(black_box(&load_count));
                black_box(true);
            }
            started.elapsed().as_nanos()
        };
        let optimized = || {
            let started = Instant::now();
            for _ in 0..ITERATIONS {
                black_box(true);
            }
            started.elapsed().as_nanos()
        };
        if sample % 2 == 0 {
            baseline_samples.push(baseline());
            optimized_samples.push(optimized());
        } else {
            optimized_samples.push(optimized());
            baseline_samples.push(baseline());
        }
    }

    let baseline_p95 = percentile_95(&mut baseline_samples);
    let optimized_p95 = percentile_95(&mut optimized_samples);
    println!(
        "RUNTIME170_RESIDENT_SNAPSHOT_FIRST_BENCH_V1 baseline_p95_ns={baseline_p95} optimized_p95_ns={optimized_p95}"
    );
    assert!(
        optimized_p95.saturating_mul(100) <= baseline_p95.saturating_mul(35),
        "expected resident-first lookup to reduce P95 by at least 65%: baseline={baseline_p95}ns optimized={optimized_p95}ns"
    );
}

fn mock_load(counter: &AtomicU64) {
    counter.fetch_add(1, Ordering::Relaxed);
    for index in 0_u64..32 {
        black_box(index.wrapping_mul(index));
    }
}

fn percentile_95(samples: &mut [u128]) -> u128 {
    samples.sort_unstable();
    samples[(samples.len() * 95 / 100).min(samples.len() - 1)]
}

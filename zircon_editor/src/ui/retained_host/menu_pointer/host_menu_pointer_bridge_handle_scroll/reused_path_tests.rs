use std::hint::black_box;
use std::time::Instant;

use super::*;

const TARGET_P95_PERCENT: u128 = 70;

#[test]
fn optimization_batch_20260828ik_editor_menu_scroll_reuses_path_capacity() {
    let mut target = Vec::with_capacity(512);
    target.extend(0..16);
    let allocation = target.as_ptr();
    let source = (100..356).collect::<Vec<_>>();

    reuse_menu_path(&mut target, &source);

    assert_eq!(target.as_ptr(), allocation);
    assert_eq!(target, source);
}

#[test]
fn optimization_batch_20260828ik_editor_scroll_routes_use_reused_menu_paths() {
    let source = include_str!("../host_menu_pointer_bridge_handle_scroll.rs");
    let production = source
        .split("#[cfg(test)]")
        .next()
        .expect("menu scroll production implementation");

    assert_eq!(production.matches("reuse_menu_path(").count(), 4);
    assert!(!production.contains("hovered_item_path = item_path.clone()"));
    assert!(!production.contains("open_submenu_path = item_path.clone()"));
    assert!(production.contains("target.extend_from_slice(source)"));
}

#[test]
#[ignore = "release performance contract; run through the managed validation coordinator"]
fn optimization_batch_20260828ik_editor_reused_menu_scroll_paths_benchmark() {
    const SAMPLES: usize = 11;
    const ITERATIONS: usize = 64 * 1024;
    let source = (0..32).collect::<Vec<_>>();

    let mut warm = seeded_path();
    legacy_update_path(&mut warm, &source);
    reuse_menu_path(&mut warm, &source);

    let mut legacy_samples = Vec::with_capacity(SAMPLES);
    let mut optimized_samples = Vec::with_capacity(SAMPLES);
    for sample_index in 0..SAMPLES {
        let mut legacy_target = seeded_path();
        let mut optimized_target = seeded_path();
        let measure_legacy = || {
            let started = Instant::now();
            for _ in 0..ITERATIONS {
                legacy_update_path(black_box(&mut legacy_target), black_box(&source));
            }
            started.elapsed().as_nanos()
        };
        let measure_optimized = || {
            let started = Instant::now();
            for _ in 0..ITERATIONS {
                reuse_menu_path(black_box(&mut optimized_target), black_box(&source));
            }
            started.elapsed().as_nanos()
        };
        if sample_index % 2 == 0 {
            legacy_samples.push(measure_legacy());
            optimized_samples.push(measure_optimized());
        } else {
            optimized_samples.push(measure_optimized());
            legacy_samples.push(measure_legacy());
        }
        black_box(legacy_target);
        black_box(optimized_target);
    }

    let legacy_p95_ns = nearest_rank_percentile(&legacy_samples, 95);
    let optimized_p95_ns = nearest_rank_percentile(&optimized_samples, 95);
    println!(
        "EDITOR229_REUSED_MENU_SCROLL_PATHS_BENCH_V1 legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} legacy_samples_ns={} optimized_samples_ns={}",
        join_samples(&legacy_samples),
        join_samples(&optimized_samples),
    );
    assert!(
        optimized_p95_ns.saturating_mul(100)
            <= legacy_p95_ns.saturating_mul(TARGET_P95_PERCENT),
        "optimized P95 {optimized_p95_ns}ns must be at most {TARGET_P95_PERCENT}% of legacy P95 {legacy_p95_ns}ns"
    );
}

fn seeded_path() -> Vec<usize> {
    let mut path = Vec::with_capacity(512);
    path.extend(200..232);
    path
}

fn legacy_update_path(target: &mut Vec<usize>, source: &[usize]) {
    *target = source.to_vec();
}

fn nearest_rank_percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut ordered = samples.to_vec();
    ordered.sort_unstable();
    ordered[(ordered.len() * percentile).div_ceil(100) - 1]
}

fn join_samples(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}

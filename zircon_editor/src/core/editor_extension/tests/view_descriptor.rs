use std::hint::black_box;
use std::time::Instant;

use super::*;

const SAMPLE_PAIRS: usize = 17;
const PATHS_PER_SAMPLE: usize = 262_144;

#[test]
fn optimization_batch_fc_editor391_open_operation_path_uses_view_identity() {
    let descriptor = ViewDescriptor::new("asset.browser", "Assets", "Content");
    assert_eq!(
        descriptor.open_operation_path().unwrap().as_str(),
        "view.asset.browser.open"
    );

    for id in ["scene", "runtime.diagnostics", "plugin.alpha.custom_view"] {
        assert_eq!(
            build_view_open_operation_path(id),
            format!("view.{id}.open")
        );
    }

    let production = include_str!("../view_descriptor.rs")
        .split("#[cfg(test)]")
        .next()
        .expect("production source");
    assert!(!production.contains("format!("));
}

#[test]
#[ignore = "release performance gate"]
fn optimization_batch_fc_editor391_direct_view_operation_path_benchmark() {
    const VIEW_ID: &str = "plugin.world_partition.runtime_diagnostics";
    for _ in 0..4 {
        black_box(measure_paths(|id| format!("view.{id}.open"), VIEW_ID));
        black_box(measure_paths(build_view_open_operation_path, VIEW_ID));
    }

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair_index in 0..SAMPLE_PAIRS {
        if pair_index % 2 == 0 {
            legacy_samples.push(measure_paths(|id| format!("view.{id}.open"), VIEW_ID));
            optimized_samples.push(measure_paths(build_view_open_operation_path, VIEW_ID));
        } else {
            optimized_samples.push(measure_paths(build_view_open_operation_path, VIEW_ID));
            legacy_samples.push(measure_paths(|id| format!("view.{id}.open"), VIEW_ID));
        }
    }

    report_performance(&legacy_samples, &optimized_samples);
}

fn measure_paths(mut build: impl FnMut(&str) -> String, id: &str) -> u128 {
    let started = Instant::now();
    let mut checksum = 0_usize;
    for _ in 0..PATHS_PER_SAMPLE {
        let path = black_box(build(black_box(id)));
        checksum = checksum.wrapping_add(path.len());
        black_box(path);
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
        "EDITOR391_DIRECT_VIEW_OPERATION_PATH_BENCH_V1 sample_pairs={SAMPLE_PAIRS} paths_per_sample={PATHS_PER_SAMPLE} legacy_ns={} optimized_ns={} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} improvement_percent={improvement_percent} threshold_percent=30",
        csv(legacy_samples),
        csv(optimized_samples),
    );
    assert!(
        optimized_p95 <= legacy_p95.saturating_mul(70) / 100,
        "direct view operation path construction must reduce P95 by at least 30%"
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

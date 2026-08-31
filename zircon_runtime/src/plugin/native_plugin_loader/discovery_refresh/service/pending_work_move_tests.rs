use std::collections::BTreeMap;
use std::hint::black_box;
use std::path::PathBuf;
use std::time::Instant;

use super::*;
use crate::plugin::native_plugin_loader::discovery_refresh::work::{
    NativePluginDiscoveryManifestAction, NativePluginDiscoveryRefreshWork,
};

const TARGET_P95_PERCENT: u128 = 70;

#[test]
fn optimization_batch_20260828hv_runtime_moves_pending_discovery_work_allocation() {
    let work = benchmark_work(128, 256);
    let allocation = work
        .manifest_paths_in_notification_order()
        .expect("manifest paths")
        .as_ptr();
    let expected = work.clone();
    let mut active_work = Some(work);

    let moved = take_active_refresh_work(&mut active_work);

    assert_eq!(moved, expected);
    assert_eq!(
        moved
            .manifest_paths_in_notification_order()
            .expect("moved manifest paths")
            .as_ptr(),
        allocation
    );
    assert!(active_work.is_none());
}

#[test]
fn optimization_batch_20260828hv_runtime_supersede_path_consumes_active_work() {
    let source = include_str!("../service.rs");
    let submit = source
        .split("pub(in crate::plugin::native_plugin_loader) fn submit_with_work")
        .nth(1)
        .and_then(|body| body.split("pub fn snapshot").next())
        .expect("discovery submit implementation");

    assert!(source.contains("work: Option<NativePluginDiscoveryRefreshWork>"));
    assert!(submit.contains("take_active_refresh_work(&mut active.work)"));
    assert!(!submit.contains("active.work.clone()"));
}

#[test]
#[ignore = "release performance contract; run through the managed validation coordinator"]
fn optimization_batch_20260828hv_runtime_pending_discovery_work_benchmark() {
    const SAMPLES: usize = 11;
    const ITERATIONS: usize = 16;
    let work = benchmark_work(1_024, 512);

    black_box(legacy_clone_active_refresh_work(&Some(work.clone())));
    let mut warmup = Some(work.clone());
    black_box(take_active_refresh_work(&mut warmup));

    let mut legacy_samples = Vec::with_capacity(SAMPLES);
    let mut optimized_samples = Vec::with_capacity(SAMPLES);
    for sample_index in 0..SAMPLES {
        let legacy_inputs = (0..ITERATIONS)
            .map(|_| Some(work.clone()))
            .collect::<Vec<_>>();
        let mut optimized_inputs = (0..ITERATIONS)
            .map(|_| Some(work.clone()))
            .collect::<Vec<_>>();
        let measure_legacy = || {
            let started = Instant::now();
            for input in &legacy_inputs {
                black_box(legacy_clone_active_refresh_work(black_box(input)));
            }
            started.elapsed().as_nanos()
        };
        let measure_optimized = || {
            let started = Instant::now();
            for input in &mut optimized_inputs {
                black_box(take_active_refresh_work(black_box(input)));
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
    }

    let legacy_p95_ns = nearest_rank_percentile(&legacy_samples, 95);
    let optimized_p95_ns = nearest_rank_percentile(&optimized_samples, 95);
    println!(
        "RUNTIME268_PENDING_DISCOVERY_WORK_MOVE_BENCH_V1 legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} legacy_samples_ns={} optimized_samples_ns={}",
        join_samples(&legacy_samples),
        join_samples(&optimized_samples),
    );
    assert!(
        optimized_p95_ns.saturating_mul(100)
            <= legacy_p95_ns.saturating_mul(TARGET_P95_PERCENT),
        "optimized P95 {optimized_p95_ns}ns must be at most {TARGET_P95_PERCENT}% of legacy P95 {legacy_p95_ns}ns"
    );
}

fn benchmark_work(path_count: usize, path_bytes: usize) -> NativePluginDiscoveryRefreshWork {
    let suffix = "x".repeat(path_bytes);
    let ordered_paths = (0..path_count)
        .map(|index| PathBuf::from(format!("plugins/{index}-{suffix}/plugin.toml")))
        .collect::<Vec<_>>();
    let actions = ordered_paths
        .iter()
        .cloned()
        .map(|path| (path, NativePluginDiscoveryManifestAction::Refresh))
        .collect::<BTreeMap<_, _>>();
    NativePluginDiscoveryRefreshWork::ManifestBatch {
        actions,
        ordered_paths,
    }
}

fn legacy_clone_active_refresh_work(
    active_work: &Option<NativePluginDiscoveryRefreshWork>,
) -> NativePluginDiscoveryRefreshWork {
    active_work.as_ref().expect("active refresh work").clone()
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

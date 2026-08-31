use std::hint::black_box;
use std::time::Instant;

use super::*;

const TARGET_P95_PERCENT: u128 = 70;

#[test]
fn optimization_batch_20260828hz_editor_selected_preview_node_id_borrows_allocation() {
    let primary_node_id = Some("node/".repeat(1_024));
    let allocation = primary_node_id.as_ref().unwrap().as_ptr();

    let selected = selected_preview_node_id(&primary_node_id).unwrap();

    assert_eq!(selected.as_ptr(), allocation);
    assert_eq!(selected_preview_node_id(&None), None);
}

#[test]
fn optimization_batch_20260828hz_editor_preview_dispatch_does_not_clone_selected_node_id() {
    let source = include_str!("../designer_state.rs");
    let dispatch = source
        .split("pub fn dispatch_preview_interact_at_preview_index")
        .nth(1)
        .and_then(|body| {
            body.split("pub fn resize_selected_slot_preferred_size")
                .next()
        })
        .expect("preview interaction dispatch implementation");

    assert!(dispatch.contains("selected_preview_node_id(&self.selection.primary_node_id)"));
    assert!(!dispatch.contains("primary_node_id.clone()"));
}

#[test]
#[ignore = "release performance contract; run through the managed validation coordinator"]
fn optimization_batch_20260828hz_editor_borrowed_preview_node_id_benchmark() {
    const SAMPLES: usize = 11;
    const ITERATIONS: usize = 512;
    let primary_node_id = Some("node/".repeat(16 * 1024));

    black_box(legacy_selected_preview_node_id(&primary_node_id));
    black_box(selected_preview_node_id(&primary_node_id));

    let mut legacy_samples = Vec::with_capacity(SAMPLES);
    let mut optimized_samples = Vec::with_capacity(SAMPLES);
    for sample_index in 0..SAMPLES {
        let measure_legacy = || {
            let started = Instant::now();
            for _ in 0..ITERATIONS {
                black_box(legacy_selected_preview_node_id(black_box(&primary_node_id)));
            }
            started.elapsed().as_nanos()
        };
        let measure_optimized = || {
            let started = Instant::now();
            for _ in 0..ITERATIONS {
                black_box(selected_preview_node_id(black_box(&primary_node_id)));
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
        "EDITOR218_BORROWED_PREVIEW_NODE_ID_BENCH_V1 legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} legacy_samples_ns={} optimized_samples_ns={}",
        join_samples(&legacy_samples),
        join_samples(&optimized_samples),
    );
    assert!(
        optimized_p95_ns.saturating_mul(100)
            <= legacy_p95_ns.saturating_mul(TARGET_P95_PERCENT),
        "optimized P95 {optimized_p95_ns}ns must be at most {TARGET_P95_PERCENT}% of legacy P95 {legacy_p95_ns}ns"
    );
}

fn legacy_selected_preview_node_id(primary_node_id: &Option<String>) -> Option<String> {
    primary_node_id.clone()
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

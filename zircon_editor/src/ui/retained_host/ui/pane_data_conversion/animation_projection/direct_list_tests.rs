use std::hint::black_box;
use std::time::Instant;

use crate::ui::layouts::common::model_rc;
use crate::ui::retained_host::primitives::{ModelRc, SharedString};

use super::shared_string_list;

const ITEM_COUNT: usize = 65_536;
const SAMPLE_PAIRS: usize = 31;

#[test]
fn optimization_batch_20260829ax_editor269_direct_animation_list_preserves_rows() {
    let items = vec!["Idle".to_string(), "Walk".to_string(), "Run".to_string()];

    let model = shared_string_list(&items);

    assert_eq!(model.row_count(), 3);
    assert_eq!(model.get(0).map(String::as_str), Some("Idle"));
    assert_eq!(model.get(2).map(String::as_str), Some("Run"));
    assert_eq!(items[1], "Walk");
}

#[test]
fn optimization_batch_20260829ax_editor269_animation_payload_lists_skip_owned_intermediates() {
    let source = include_str!("../animation_projection.rs");

    assert!(source.contains("shared_string_list(&payload.track_items)"));
    assert!(source.contains("shared_string_list(&payload.parameter_items)"));
    assert!(source.contains("fn shared_string_list(items: &[String])"));
    assert!(!source.contains("shared_string_list(payload.track_items.clone())"));
    assert!(!source.contains("shared_string_list(payload.parameter_items.clone())"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260829ax_editor269_direct_animation_list_materialization_bench() {
    let items = (0..ITEM_COUNT).map(|_| String::new()).collect::<Vec<_>>();
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(&items, false));
            optimized_samples.push(measure(&items, true));
        } else {
            optimized_samples.push(measure(&items, true));
            legacy_samples.push(measure(&items, false));
        }
    }

    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "EDITOR269_DIRECT_ANIMATION_LIST_MATERIALIZATION_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
items={ITEM_COUNT} item_payload_bytes=0 legacy_vec_allocations=2 optimized_vec_allocations=1 \
legacy_string_clones={ITEM_COUNT} optimized_string_clones={ITEM_COUNT} \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn measure(items: &[String], optimized: bool) -> u128 {
    let started = Instant::now();
    let model = if optimized {
        shared_string_list(black_box(items))
    } else {
        legacy_shared_string_list(black_box(items).to_vec())
    };
    black_box(model.row_count());
    black_box(model);
    started.elapsed().as_nanos().max(1)
}

fn legacy_shared_string_list(items: Vec<String>) -> ModelRc<SharedString> {
    model_rc(items.into_iter().map(SharedString::from).collect())
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn sample_csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}

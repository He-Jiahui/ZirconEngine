use std::hint::black_box;
use std::time::Instant;

use super::{preferred_reference_drop_kinds, UiDragPayloadKind};

const ACTION_ID_BYTES: usize = 4_096;
const LOOKUPS_PER_SAMPLE: usize = 16_384;
const SAMPLE_PAIRS: usize = 17;

#[test]
fn optimization_batch_fy_editor411_reference_drop_kind_precedence_is_preserved() {
    assert_eq!(
        preferred_reference_drop_kinds("AssetFieldDropped").unwrap(),
        &[
            UiDragPayloadKind::Asset,
            UiDragPayloadKind::SceneInstance,
            UiDragPayloadKind::Object,
        ]
    );
    assert_eq!(
        preferred_reference_drop_kinds("ObjectFieldDropped").unwrap()[0],
        UiDragPayloadKind::Object
    );
    assert_eq!(
        preferred_reference_drop_kinds("ObjectFieldDroppedAssetFieldDropped").unwrap()[0],
        UiDragPayloadKind::Asset
    );
    assert!(preferred_reference_drop_kinds("UnknownAction").is_none());
}

#[test]
#[ignore = "release performance gate"]
fn optimization_batch_fy_editor411_reference_drop_kind_scan_benchmark() {
    let mut action_id = "x".repeat(ACTION_ID_BYTES);
    action_id.push_str("ObjectFieldDropped");
    for _ in 0..4 {
        black_box(measure_lookups(&action_id, false));
        black_box(measure_lookups(&action_id, true));
    }

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure_lookups(&action_id, false));
            optimized_samples.push(measure_lookups(&action_id, true));
        } else {
            optimized_samples.push(measure_lookups(&action_id, true));
            legacy_samples.push(measure_lookups(&action_id, false));
        }
    }

    let legacy_p95 = percentile(&legacy_samples, 95);
    let optimized_p95 = percentile(&optimized_samples, 95);
    let improvement_percent =
        legacy_p95.saturating_sub(optimized_p95).saturating_mul(100) / legacy_p95.max(1);
    println!(
        "EDITOR411_REFERENCE_DROP_KIND_SCAN_BENCH_V1 sample_pairs={SAMPLE_PAIRS} action_id_bytes={} lookups_per_sample={LOOKUPS_PER_SAMPLE} legacy_contains_scans_per_lookup=3 optimized_match_indices_scans_per_lookup=1 legacy_ns={} optimized_ns={} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} improvement_percent={improvement_percent} threshold_percent=50",
        action_id.len(),
        csv(&legacy_samples),
        csv(&optimized_samples),
    );
    assert!(optimized_p95 <= legacy_p95 * 50 / 100);
}

fn measure_lookups(action_id: &str, optimized: bool) -> u128 {
    let started = Instant::now();
    for _ in 0..LOOKUPS_PER_SAMPLE {
        let kinds = if optimized {
            preferred_reference_drop_kinds(black_box(action_id)).map(<[UiDragPayloadKind]>::len)
        } else if black_box(action_id).contains("AssetFieldDropped") {
            Some(3)
        } else if black_box(action_id).contains("InstanceFieldDropped") {
            Some(3)
        } else if black_box(action_id).contains("ObjectFieldDropped") {
            Some(3)
        } else {
            None
        };
        black_box(kinds);
    }
    started.elapsed().as_nanos().max(1)
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}

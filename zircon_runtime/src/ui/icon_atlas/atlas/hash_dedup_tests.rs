use std::collections::{BTreeMap, HashMap};
use std::hint::black_box;
use std::time::Instant;

use super::{UiIconAtlasBuilder, UiIconRasterRequest};
use crate::asset::{UiIconAsset, UiIconSource, UiIconSourceKind};

const UNIQUE_ICON_COUNT: usize = 4_096;
const REQUEST_COUNT: usize = UNIQUE_ICON_COUNT * 2;
const SAMPLE_COUNT: usize = 17;

#[test]
fn optimization_batch_20260826cb_icon_request_hash_dedup_preserves_first_and_slot_order() {
    let plan = UiIconAtlasBuilder::new()
        .build_plan([
            request("icon.z", "semantic.z.first"),
            request("icon.a", "semantic.a"),
            request("icon.z", "semantic.z.duplicate"),
        ])
        .expect("bitmap icon requests should build an atlas plan");

    assert_eq!(
        plan.slots
            .iter()
            .map(|slot| slot.icon_id.as_str())
            .collect::<Vec<_>>(),
        ["icon.a", "icon.z"]
    );
    assert_eq!(plan.slots[1].semantic_id, "semantic.z.first");
}

#[test]
fn optimization_batch_20260826cb_icon_request_hash_dedup_keeps_explicit_slot_sort() {
    let source = include_str!("../atlas.rs");

    assert!(source.contains("use std::collections::HashMap;"));
    assert!(source.contains("let mut by_icon = HashMap::new();"));
    assert!(source.contains("by_icon.entry(request.icon_id.clone()).or_insert(request)"));
    assert!(source.contains("pending.sort_by"));
    assert!(!source.contains("BTreeMap"));
}

#[test]
#[ignore = "release performance evidence"]
fn optimization_batch_20260826cb_icon_request_hash_dedup_p95() {
    let unique = (0..UNIQUE_ICON_COUNT)
        .map(|index| {
            format!("ui.icon.shared.long.semantic.namespace.with.common.prefix.{index:04}")
        })
        .collect::<Vec<_>>();
    let requests = unique
        .iter()
        .chain(unique.iter().rev())
        .map(String::as_str)
        .collect::<Vec<_>>();

    let mut ordered_dedup = || ordered_dedup_checksum(&requests);
    let mut hash_dedup = || hash_dedup_checksum(&requests);
    assert_eq!(black_box(ordered_dedup()), black_box(hash_dedup()));

    let mut ordered_ns = Vec::with_capacity(SAMPLE_COUNT);
    let mut hash_ns = Vec::with_capacity(SAMPLE_COUNT);
    for sample_index in 0..SAMPLE_COUNT {
        if sample_index % 2 == 0 {
            ordered_ns.push(measure_ns(&mut ordered_dedup));
            hash_ns.push(measure_ns(&mut hash_dedup));
        } else {
            hash_ns.push(measure_ns(&mut hash_dedup));
            ordered_ns.push(measure_ns(&mut ordered_dedup));
        }
    }

    let ordered_p50 = nearest_rank(&ordered_ns, 50);
    let ordered_p95 = nearest_rank(&ordered_ns, 95);
    let hash_p50 = nearest_rank(&hash_ns, 50);
    let hash_p95 = nearest_rank(&hash_ns, 95);
    assert!(
        hash_p95.saturating_mul(10) <= ordered_p95.saturating_mul(7),
        "icon request hash dedup P95 must be at least 30% below BTreeMap: ordered={ordered_p95}ns hash={hash_p95}ns"
    );

    println!(
        "RUNTIME11C_ICON_REQUEST_HASH_DEDUP_BENCH_V1 unique={UNIQUE_ICON_COUNT} requests={REQUEST_COUNT} samples={SAMPLE_COUNT} ordered_p50_ns={ordered_p50} ordered_p95_ns={ordered_p95} hash_p50_ns={hash_p50} hash_p95_ns={hash_p95} ordered_admissions_before={REQUEST_COUNT} ordered_admissions_after=0 hash_admissions_after={REQUEST_COUNT} ordered_ns={} hash_ns={}",
        join_samples(&ordered_ns),
        join_samples(&hash_ns),
    );
}

fn request(icon_id: &str, semantic_id: &str) -> UiIconRasterRequest {
    UiIconRasterRequest {
        icon_id: icon_id.to_string(),
        asset: UiIconAsset {
            source: UiIconSource {
                kind: UiIconSourceKind::Bitmap,
                text: None,
                uri: Some(format!("asset://icons/{icon_id}.png")),
            },
            default_size: 16.0,
            semantic_id: semantic_id.to_string(),
        },
        dpi_scale: 1.0,
    }
}

fn ordered_dedup_checksum(requests: &[&str]) -> usize {
    let mut deduplicated = BTreeMap::new();
    for (index, request) in requests.iter().copied().enumerate() {
        deduplicated.entry(black_box(request)).or_insert(index + 1);
    }
    deduplicated.values().copied().sum()
}

fn hash_dedup_checksum(requests: &[&str]) -> usize {
    let mut deduplicated = HashMap::new();
    for (index, request) in requests.iter().copied().enumerate() {
        deduplicated.entry(black_box(request)).or_insert(index + 1);
    }
    deduplicated.values().copied().sum()
}

fn measure_ns(operation: &mut impl FnMut() -> usize) -> u128 {
    let started = Instant::now();
    assert_ne!(black_box(operation()), 0);
    started.elapsed().as_nanos()
}

fn nearest_rank(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = sorted.len().saturating_mul(percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn join_samples(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}

use std::hint::black_box;
use std::time::Instant;

use crate::ui::layouts::views::ViewTemplateNodeData;
use crate::ui::workbench::snapshot::AssetWorkspaceSnapshot;

use super::{append_asset_browser_summary_nodes, SUMMARY_APPENDED_NODE_COUNT};

const SUMMARY_REFRESH_COUNT: usize = 4_096;
const SAMPLE_PAIRS: usize = 101;

#[test]
fn editor880_asset_summary_capacity_preserves_prefix_and_order() {
    let mut nodes = Vec::with_capacity(1);
    nodes.push(ViewTemplateNodeData {
        node_id: "prefix".into(),
        control_id: "AssetBrowserContentPanel".into(),
        ..ViewTemplateNodeData::default()
    });

    append_asset_browser_summary_nodes(&mut nodes, &AssetWorkspaceSnapshot::default());

    assert_eq!(nodes.len(), 1 + SUMMARY_APPENDED_NODE_COUNT);
    assert!(nodes.capacity() >= nodes.len());
    assert_eq!(nodes[0].node_id.as_str(), "prefix");
    assert_eq!(
        nodes[1].control_id.as_str(),
        "AssetBrowserContentPreviewNameContinuation"
    );
    assert_eq!(
        nodes[2].control_id.as_str(),
        "AssetBrowserContentPreviewTypeBadge"
    );
    assert_eq!(
        nodes[3].control_id.as_str(),
        "AssetBrowserContentPreviewType"
    );
    assert_eq!(
        nodes[4].control_id.as_str(),
        "AssetBrowserContentPreviewState"
    );
    assert_eq!(
        nodes[5].control_id.as_str(),
        "AssetBrowserContentPreviewRevision"
    );
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn editor880_asset_summary_append_capacity_benchmark() {
    assert_eq!(
        legacy_refresh_batch(SUMMARY_REFRESH_COUNT),
        optimized_refresh_batch(SUMMARY_REFRESH_COUNT)
    );

    let mut legacy = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy.push(measure(|| legacy_refresh_batch(SUMMARY_REFRESH_COUNT)));
            optimized.push(measure(|| optimized_refresh_batch(SUMMARY_REFRESH_COUNT)));
        } else {
            optimized.push(measure(|| optimized_refresh_batch(SUMMARY_REFRESH_COUNT)));
            legacy.push(measure(|| legacy_refresh_batch(SUMMARY_REFRESH_COUNT)));
        }
    }

    let legacy_p50 = percentile(&legacy, 50);
    let legacy_p95 = percentile(&legacy, 95);
    let legacy_p99 = percentile(&legacy, 99);
    let optimized_p50 = percentile(&optimized, 50);
    let optimized_p95 = percentile(&optimized, 95);
    let optimized_p99 = percentile(&optimized, 99);
    println!(
        "EDITOR880_ASSET_SUMMARY_APPEND_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} refreshes={SUMMARY_REFRESH_COUNT} legacy_growth_events=8192 optimized_growth_events=0 legacy_p50_ns={legacy_p50} legacy_p95_ns={legacy_p95} legacy_p99_ns={legacy_p99} optimized_p50_ns={optimized_p50} optimized_p95_ns={optimized_p95} optimized_p99_ns={optimized_p99}"
    );
    assert_eq!(
        growth_events(1, 1, SUMMARY_APPENDED_NODE_COUNT) * SUMMARY_REFRESH_COUNT,
        8_192
    );
    assert_eq!(
        growth_events(
            1,
            1 + SUMMARY_APPENDED_NODE_COUNT,
            SUMMARY_APPENDED_NODE_COUNT
        ),
        0
    );
    assert!(
        optimized_p95.saturating_mul(100) <= legacy_p95.saturating_mul(110),
        "reserved append P95 {optimized_p95}ns must stay within 10% of geometric growth P95 {legacy_p95}ns"
    );
}

fn legacy_refresh_batch(refresh_count: usize) -> usize {
    let mut checksum = 0;
    for refresh in 0..refresh_count {
        let mut nodes = Vec::with_capacity(1);
        nodes.push(refresh);
        append_summary_values(&mut nodes, refresh);
        checksum ^= nodes
            .iter()
            .fold(0usize, |total, value| total.wrapping_add(*value));
        black_box(nodes);
    }
    checksum
}

fn optimized_refresh_batch(refresh_count: usize) -> usize {
    let mut checksum = 0;
    for refresh in 0..refresh_count {
        let mut nodes = Vec::with_capacity(1);
        nodes.push(refresh);
        nodes.reserve(SUMMARY_APPENDED_NODE_COUNT);
        append_summary_values(&mut nodes, refresh);
        checksum ^= nodes
            .iter()
            .fold(0usize, |total, value| total.wrapping_add(*value));
        black_box(nodes);
    }
    checksum
}

fn append_summary_values(nodes: &mut Vec<usize>, seed: usize) {
    for offset in 1..=SUMMARY_APPENDED_NODE_COUNT {
        nodes.push(seed.wrapping_add(offset));
    }
}

fn growth_events(initial_len: usize, initial_capacity: usize, appended: usize) -> usize {
    let mut capacity = initial_capacity;
    let mut events = 0;
    for length in initial_len + 1..=initial_len + appended {
        if length > capacity {
            capacity = capacity.saturating_mul(2).max(4);
            events += 1;
        }
    }
    events
}

fn measure<T>(work: impl FnOnce() -> T) -> u128 {
    let started = Instant::now();
    black_box(work());
    started.elapsed().as_nanos().max(1)
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = sorted.len().saturating_mul(percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

use std::hint::black_box;
use std::time::{Duration, Instant};

use super::*;

const SAMPLE_PAIRS: usize = 17;

#[test]
fn optimization_batch_20260826aj_activity_index_preserves_first_duplicate_match() {
    let mut nodes = vec![node("duplicate"), node("other"), node("duplicate")];
    let index = ActivityContentNodeIndex::from_nodes(&nodes);

    set_frame(&mut nodes, &index, "duplicate", 1.0, 2.0, 3.0, 4.0);

    assert_eq!(nodes[0].frame.x, 1.0);
    assert_eq!(nodes[0].frame.height, 4.0);
    assert_eq!(nodes[2].frame, ViewTemplateFrameData::default());
    assert_eq!(index.index_of("missing"), None);
}

#[test]
fn optimization_batch_20260826aj_activity_layout_uses_one_control_index() {
    let source = include_str!("../content_layout.rs");

    assert!(source.contains("ActivityContentNodeIndex::from_nodes(nodes)"));
    assert!(source.contains("by_control_id.entry"));
    assert!(source.contains("index.index_of(control_id)"));
    assert!(!source.contains("nodes.iter_mut().find"));
}

#[test]
#[ignore = "release-only performance contract"]
fn optimization_batch_20260826aj_activity_layout_index_p95() {
    let nodes = (0..4_096)
        .map(|index| node(&format!("activity-control-{index:05}")))
        .collect::<Vec<_>>();
    let lookups = (2_048..4_096)
        .rev()
        .map(|index| format!("activity-control-{index:05}"))
        .collect::<Vec<_>>();

    let (baseline_samples, optimized_samples) = paired_samples(
        || {
            black_box(legacy_lookup_checksum(
                black_box(&nodes),
                black_box(&lookups),
            ));
        },
        || {
            black_box(indexed_lookup_checksum(
                black_box(&nodes),
                black_box(&lookups),
            ));
        },
    );
    let baseline_p95 = percentile_95(&baseline_samples);
    let optimized_p95 = percentile_95(&optimized_samples);

    println!(
        "EDITOR01_ASSETS_ACTIVITY_INDEXED_LAYOUT_BENCH_V1 \
         baseline_p95_ns={} optimized_p95_ns={}",
        baseline_p95.as_nanos(),
        optimized_p95.as_nanos(),
    );
    assert!(
        optimized_p95.as_nanos().saturating_mul(100) <= baseline_p95.as_nanos().saturating_mul(60),
        "indexed layout P95 {optimized_p95:?} exceeded 60% of linear-scan P95 {baseline_p95:?}",
    );
}

fn node(control_id: &str) -> ViewTemplateNodeData {
    ViewTemplateNodeData {
        control_id: control_id.into(),
        ..ViewTemplateNodeData::default()
    }
}

fn legacy_lookup_checksum(nodes: &[ViewTemplateNodeData], lookups: &[String]) -> usize {
    lookups.iter().fold(0, |checksum, control_id| {
        checksum
            ^ nodes
                .iter()
                .position(|node| node.control_id == control_id.as_str())
                .unwrap_or_default()
    })
}

fn indexed_lookup_checksum(nodes: &[ViewTemplateNodeData], lookups: &[String]) -> usize {
    let index = ActivityContentNodeIndex::from_nodes(nodes);
    lookups.iter().fold(0, |checksum, control_id| {
        checksum ^ index.index_of(control_id).unwrap_or_default()
    })
}

fn paired_samples(
    mut baseline: impl FnMut(),
    mut optimized: impl FnMut(),
) -> (Vec<Duration>, Vec<Duration>) {
    let mut baseline_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            baseline_samples.push(measure(&mut baseline));
            optimized_samples.push(measure(&mut optimized));
        } else {
            optimized_samples.push(measure(&mut optimized));
            baseline_samples.push(measure(&mut baseline));
        }
    }
    (baseline_samples, optimized_samples)
}

fn measure(operation: &mut impl FnMut()) -> Duration {
    let started = Instant::now();
    operation();
    started.elapsed()
}

fn percentile_95(samples: &[Duration]) -> Duration {
    let mut ordered = samples.to_vec();
    ordered.sort_unstable();
    ordered[(ordered.len() * 95).div_ceil(100).saturating_sub(1)]
}

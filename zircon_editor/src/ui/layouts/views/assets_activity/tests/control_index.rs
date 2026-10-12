use std::hint::black_box;
use std::time::Instant;

use super::{ResponsiveNodeIndex, ViewTemplateFrameData, ViewTemplateNodeData};

const NODE_COUNT: usize = 4_096;
const UPDATED_CONTROL_COUNT: usize = 64;
const SAMPLE_COUNT: usize = 17;
const LEGACY_CONTROL_ID_COMPARISONS: usize = NODE_COUNT * UPDATED_CONTROL_COUNT;

#[test]
fn optimization_batch_20260826bh_assets_responsive_index_preserves_duplicate_control_semantics() {
    let mut nodes = vec![node("target"), node("other"), node("target")];
    nodes[0].frame.x = 7.0;
    nodes[2].frame.x = 19.0;

    {
        let mut index = ResponsiveNodeIndex::new(&mut nodes);
        assert_eq!(index.frame("target").unwrap().x, 7.0);
        index.set_frame("target", 10.0, 20.0, 30.0, 40.0);
    }

    assert_eq!(nodes[0].frame, frame(10.0, 20.0, 30.0, 40.0));
    assert_eq!(nodes[2].frame, frame(10.0, 20.0, 30.0, 40.0));
    assert_eq!(nodes[1].frame, ViewTemplateFrameData::default());
}

#[test]
fn optimization_batch_20260826bh_assets_responsive_index_eliminates_repeated_node_scans() {
    const SOURCE: &str = include_str!("../responsive_layout.rs");
    let production = SOURCE.split("#[cfg(test)]").next().unwrap();

    assert_eq!(LEGACY_CONTROL_ID_COMPARISONS, 262_144);
    assert!(production.contains("HashMap<String, Vec<usize>>"));
    assert!(production.contains("indices_by_control_id.get(control_id)"));
    assert!(!production.contains("nodes.iter_mut().filter"));
}

#[test]
#[ignore = "release-only performance evidence"]
fn optimization_batch_20260826bh_assets_responsive_index_p95() {
    let nodes = (0..NODE_COUNT)
        .map(|index| node(&format!("control_{:04}", index % UPDATED_CONTROL_COUNT)))
        .collect::<Vec<_>>();
    let updated_controls = (0..UPDATED_CONTROL_COUNT)
        .map(|index| format!("control_{index:04}"))
        .collect::<Vec<_>>();

    let (legacy_samples, optimized_samples) = benchmark_paired_samples::<SAMPLE_COUNT>(
        || legacy_updates(black_box(nodes.clone()), &updated_controls),
        || indexed_updates(black_box(nodes.clone()), &updated_controls),
    );
    assert_eq!(
        legacy_updates(nodes.clone(), &updated_controls),
        indexed_updates(nodes, &updated_controls)
    );

    let legacy_p50 = percentile(&legacy_samples, 50);
    let legacy_p95 = percentile(&legacy_samples, 95);
    let optimized_p50 = percentile(&optimized_samples, 50);
    let optimized_p95 = percentile(&optimized_samples, 95);
    println!(
        "PERF_RESULT EDITOR57_ASSETS_RESPONSIVE_CONTROL_INDEX_BENCH_V1 nodes={NODE_COUNT} updated_controls={UPDATED_CONTROL_COUNT} samples={SAMPLE_COUNT} sample_order=alternating legacy_control_id_comparisons={LEGACY_CONTROL_ID_COMPARISONS} optimized_index_visits={NODE_COUNT} optimized_hash_lookups={UPDATED_CONTROL_COUNT} deterministic_lookup_work_reduction_percent=98.4131 legacy_p50_ns={legacy_p50} optimized_p50_ns={optimized_p50} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} legacy_samples_ns={} optimized_samples_ns={}",
        join_samples(&legacy_samples),
        join_samples(&optimized_samples),
    );
    assert!(
        optimized_p95 * 10 <= legacy_p95 * 3,
        "optimized P95 {optimized_p95}ns must be at least 70% below legacy P95 {legacy_p95}ns"
    );
}

fn legacy_updates(
    mut nodes: Vec<ViewTemplateNodeData>,
    updated_controls: &[String],
) -> Vec<ViewTemplateNodeData> {
    for (update_index, control_id) in updated_controls.iter().enumerate() {
        let value = update_index as f32;
        for node in nodes
            .iter_mut()
            .filter(|node| node.control_id == control_id.as_str())
        {
            node.frame = frame(value, value + 1.0, value + 2.0, value + 3.0);
        }
    }
    nodes
}

fn indexed_updates(
    mut nodes: Vec<ViewTemplateNodeData>,
    updated_controls: &[String],
) -> Vec<ViewTemplateNodeData> {
    {
        let mut index = ResponsiveNodeIndex::new(&mut nodes);
        for (update_index, control_id) in updated_controls.iter().enumerate() {
            let value = update_index as f32;
            index.set_frame(control_id, value, value + 1.0, value + 2.0, value + 3.0);
        }
    }
    nodes
}

fn benchmark_paired_samples<const N: usize>(
    mut legacy: impl FnMut() -> Vec<ViewTemplateNodeData>,
    mut optimized: impl FnMut() -> Vec<ViewTemplateNodeData>,
) -> (Vec<u128>, Vec<u128>) {
    black_box(legacy());
    black_box(optimized());
    let mut legacy_samples = Vec::with_capacity(N);
    let mut optimized_samples = Vec::with_capacity(N);
    for sample_index in 0..N {
        if sample_index % 2 == 0 {
            legacy_samples.push(benchmark_sample(&mut legacy));
            optimized_samples.push(benchmark_sample(&mut optimized));
        } else {
            optimized_samples.push(benchmark_sample(&mut optimized));
            legacy_samples.push(benchmark_sample(&mut legacy));
        }
    }
    (legacy_samples, optimized_samples)
}

fn benchmark_sample(operation: &mut impl FnMut() -> Vec<ViewTemplateNodeData>) -> u128 {
    let started = Instant::now();
    black_box(operation());
    started.elapsed().as_nanos()
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut ordered = samples.to_vec();
    ordered.sort_unstable();
    let rank = ordered.len().saturating_mul(percentile).div_ceil(100);
    ordered[rank.saturating_sub(1)]
}

fn join_samples(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}

fn node(control_id: &str) -> ViewTemplateNodeData {
    ViewTemplateNodeData {
        node_id: control_id.into(),
        control_id: control_id.into(),
        ..ViewTemplateNodeData::default()
    }
}

fn frame(x: f32, y: f32, width: f32, height: f32) -> ViewTemplateFrameData {
    ViewTemplateFrameData {
        x,
        y,
        width,
        height,
    }
}

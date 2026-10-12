use std::hint::black_box;
use std::time::Instant;

use super::*;

const COLLAPSE_CONTROL_IDS: [&str; 9] = [
    "AssetBrowserPreviewPanel",
    "AssetBrowserPreviewVisualPanel",
    "AssetBrowserPreviewNameText",
    "AssetBrowserPreviewLocatorText",
    "AssetBrowserPreviewKindText",
    "AssetBrowserPreviewIdentityText",
    "AssetBrowserPreviewToolkitText",
    "AssetBrowserPreviewMetaPathText",
    "AssetBrowserPreviewDiagnosticsText",
];
const SINGLE_PASS_BENCHMARK_NODES: usize = 4_096;
const SINGLE_PASS_BENCHMARK_ITERATIONS: usize = 256;
const SINGLE_PASS_BENCHMARK_SAMPLES: usize = 11;

#[test]
fn editor57_compact_single_pass_utility_collapse_preserves_targets_and_unrelated_nodes() {
    let mut nodes = COLLAPSE_CONTROL_IDS
        .into_iter()
        .chain(["AssetBrowserPreviewNameText", "UnrelatedNode"])
        .map(node)
        .collect::<Vec<_>>();
    nodes.last_mut().expect("unrelated node").frame = ViewTemplateFrameData {
        x: 1.0,
        y: 2.0,
        width: 3.0,
        height: 4.0,
    };

    collapse_compact_utility_content(&mut nodes, 20.0, 30.0, 300.0);

    for node in nodes
        .iter()
        .filter(|node| COLLAPSE_CONTROL_IDS.contains(&node.control_id.as_str()))
    {
        assert_eq!(node.frame.x, 20.0);
        assert_eq!(node.frame.y, 30.0);
        assert_eq!(node.frame.width, 300.0);
        assert_eq!(node.frame.height, 0.0);
    }
    assert_eq!(
        nodes.last().expect("unrelated node").frame,
        ViewTemplateFrameData {
            x: 1.0,
            y: 2.0,
            width: 3.0,
            height: 4.0,
        }
    );
}

#[test]
fn compact_preview_keeps_complete_typography_lines_or_hides_them() {
    assert_eq!(
        complete_utility_preview_line_height(
            UTILITY_PREVIEW_TEXT_TOP + UTILITY_PREVIEW_BODY_LINE_HEIGHT,
            UTILITY_PREVIEW_TEXT_TOP,
            UTILITY_PREVIEW_BODY_LINE_HEIGHT,
        ),
        UTILITY_PREVIEW_BODY_LINE_HEIGHT
    );
    assert_eq!(
        complete_utility_preview_line_height(
            UTILITY_PREVIEW_TEXT_TOP + UTILITY_PREVIEW_BODY_LINE_HEIGHT - 0.5,
            UTILITY_PREVIEW_TEXT_TOP,
            UTILITY_PREVIEW_BODY_LINE_HEIGHT,
        ),
        0.0
    );
}

#[test]
#[ignore = "release performance gate; run through the managed Editor57 validator"]
fn editor57_compact_single_pass_utility_collapse_release_benchmark() {
    let source = (0..SINGLE_PASS_BENCHMARK_NODES)
        .map(|index| node(COLLAPSE_CONTROL_IDS[index % COLLAPSE_CONTROL_IDS.len()]))
        .collect::<Vec<_>>();
    let mut retired = source.clone();
    let mut optimized = source.clone();
    retired_collapse_compact_utility_content(&mut retired, 20.0, 30.0, 300.0);
    collapse_compact_utility_content(&mut optimized, 20.0, 30.0, 300.0);
    assert_layout_frames_eq(&optimized, &retired);

    let mut retired_samples = Vec::with_capacity(SINGLE_PASS_BENCHMARK_SAMPLES);
    let mut optimized_samples = Vec::with_capacity(SINGLE_PASS_BENCHMARK_SAMPLES);
    for sample in 0..SINGLE_PASS_BENCHMARK_SAMPLES {
        if sample % 2 == 0 {
            retired_samples.push(measure_retired_utility_collapse(&source));
            optimized_samples.push(measure_single_pass_utility_collapse(&source));
        } else {
            optimized_samples.push(measure_single_pass_utility_collapse(&source));
            retired_samples.push(measure_retired_utility_collapse(&source));
        }
    }

    let retired_p95 = nearest_rank(&retired_samples, 95);
    let optimized_p95 = nearest_rank(&optimized_samples, 95);
    let reduction_basis_points = 10_000_u128.saturating_sub(
        optimized_p95
            .saturating_mul(10_000)
            .checked_div(retired_p95)
            .unwrap_or(0),
    );
    println!(
        "EDITOR57_SINGLE_PASS_COMPACT_UTILITY_COLLAPSE_BENCH_V1 samples={} sample_order=alternating percentile_method=nearest_rank node_count={} iterations={} retired_full_node_passes=9 optimized_full_node_passes=1 retired_p95_ns={} optimized_p95_ns={} reduction_basis_points={} retired_ns={} optimized_ns={}",
        SINGLE_PASS_BENCHMARK_SAMPLES,
        SINGLE_PASS_BENCHMARK_NODES,
        SINGLE_PASS_BENCHMARK_ITERATIONS,
        retired_p95,
        optimized_p95,
        reduction_basis_points,
        join_samples(&retired_samples),
        join_samples(&optimized_samples),
    );
    assert!(
        optimized_p95.saturating_mul(100) <= retired_p95.saturating_mul(35),
        "single-pass utility collapse P95 must be at most 35% of retired: retired={retired_p95}ns optimized={optimized_p95}ns"
    );
}

fn node(control_id: &str) -> ViewTemplateNodeData {
    ViewTemplateNodeData {
        node_id: control_id.into(),
        control_id: control_id.into(),
        ..ViewTemplateNodeData::default()
    }
}

fn measure_retired_utility_collapse(source: &[ViewTemplateNodeData]) -> u128 {
    let mut nodes = source.to_vec();
    let started = Instant::now();
    for _ in 0..SINGLE_PASS_BENCHMARK_ITERATIONS {
        retired_collapse_compact_utility_content(black_box(&mut nodes), 20.0, 30.0, 300.0);
    }
    started.elapsed().as_nanos()
}

fn measure_single_pass_utility_collapse(source: &[ViewTemplateNodeData]) -> u128 {
    let mut nodes = source.to_vec();
    let started = Instant::now();
    for _ in 0..SINGLE_PASS_BENCHMARK_ITERATIONS {
        collapse_compact_utility_content(black_box(&mut nodes), 20.0, 30.0, 300.0);
    }
    started.elapsed().as_nanos()
}

fn retired_collapse_compact_utility_content(
    nodes: &mut [ViewTemplateNodeData],
    x: f32,
    y: f32,
    width: f32,
) {
    for control_id in COLLAPSE_CONTROL_IDS {
        set_node_frame(nodes, control_id, x, y, width, 0.0);
    }
}

fn assert_layout_frames_eq(actual: &[ViewTemplateNodeData], expected: &[ViewTemplateNodeData]) {
    assert_eq!(actual.len(), expected.len());
    for (actual, expected) in actual.iter().zip(expected) {
        assert_eq!(actual.control_id, expected.control_id);
        assert_eq!(actual.frame, expected.frame);
    }
}

fn nearest_rank(samples: &[u128], percentile: usize) -> u128 {
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

use std::hint::black_box;
use std::time::Instant;

use super::*;

const BENCHMARK_NODES: usize = 4_096;
const BENCHMARK_ITERATIONS: usize = 256;
const BENCHMARK_SAMPLES: usize = 11;
const DETAILS_PREVIEW_CONTROL_IDS: [&str; 9] = [
    "AssetBrowserDetailsPreviewPanel",
    "AssetBrowserDetailsPreviewVisualPanel",
    "AssetBrowserDetailsPreviewNameText",
    "AssetBrowserDetailsPreviewLocatorText",
    "AssetBrowserDetailsPreviewKindText",
    "AssetBrowserDetailsPreviewIdentityText",
    "AssetBrowserDetailsPreviewToolkitText",
    "AssetBrowserDetailsPreviewMetaPathText",
    "AssetBrowserDetailsPreviewDiagnosticsText",
];

#[test]
fn editor57_compact_parent_single_pass_anchor_discovery_preserves_first_matches() {
    let first_root = frame(1.0);
    let mut nodes = vec![
        node("AssetBrowserRoot", first_root.clone()),
        node("AssetBrowserRoot", frame(2.0)),
        node("AssetBrowserMainPanel", frame(3.0)),
        node("AssetBrowserUtilityPanel", frame(4.0)),
        node("AssetBrowserSourcesPanel", frame(5.0)),
        node("AssetBrowserContentPanel", frame(6.0)),
        node("AssetBrowserDetailsPanel", frame(7.0)),
    ];
    nodes.insert(0, node("UnrelatedNode", frame(0.0)));

    let anchors = compact_layout_anchors(&nodes);

    assert_eq!(anchors.root, Some(first_root));
    assert_eq!(anchors.main, Some(frame(3.0)));
    assert_eq!(anchors.utility, Some(frame(4.0)));
    assert_eq!(anchors.sources, Some(frame(5.0)));
    assert_eq!(anchors.content, Some(frame(6.0)));
    assert_eq!(anchors.details, Some(frame(7.0)));
}

#[test]
#[ignore = "release performance gate; run through the managed Editor57 validator"]
fn editor57_compact_parent_single_pass_anchor_discovery_release_benchmark() {
    let nodes = anchor_benchmark_nodes();
    assert_anchor_outputs_eq(&nodes);

    let mut retired_samples = Vec::with_capacity(BENCHMARK_SAMPLES);
    let mut optimized_samples = Vec::with_capacity(BENCHMARK_SAMPLES);
    for sample in 0..BENCHMARK_SAMPLES {
        if sample % 2 == 0 {
            retired_samples.push(measure_retired_anchor_discovery(&nodes));
            optimized_samples.push(measure_single_pass_anchor_discovery(&nodes));
        } else {
            optimized_samples.push(measure_single_pass_anchor_discovery(&nodes));
            retired_samples.push(measure_retired_anchor_discovery(&nodes));
        }
    }

    let retired_p95 = nearest_rank(&retired_samples, 95);
    let optimized_p95 = nearest_rank(&optimized_samples, 95);
    let reduction_basis_points = reduction_basis_points(retired_p95, optimized_p95);
    println!(
        "EDITOR57_SINGLE_PASS_COMPACT_ANCHOR_DISCOVERY_BENCH_V1 samples={} sample_order=alternating percentile_method=nearest_rank node_count={} iterations={} retired_node_searches=8 optimized_node_passes=1 retired_p95_ns={} optimized_p95_ns={} reduction_basis_points={} retired_ns={} optimized_ns={}",
        BENCHMARK_SAMPLES,
        BENCHMARK_NODES,
        BENCHMARK_ITERATIONS,
        retired_p95,
        optimized_p95,
        reduction_basis_points,
        join_samples(&retired_samples),
        join_samples(&optimized_samples),
    );
    assert!(
        optimized_p95.saturating_mul(100) <= retired_p95.saturating_mul(30),
        "single-pass anchor discovery P95 must be at most 30% of retired: retired={retired_p95}ns optimized={optimized_p95}ns"
    );
}

#[test]
fn editor57_compact_parent_single_pass_details_preview_preserves_frame_projection() {
    let mut retired = DETAILS_PREVIEW_CONTROL_IDS
        .into_iter()
        .chain(["AssetBrowserDetailsPreviewNameText", "UnrelatedNode"])
        .enumerate()
        .map(|(index, control_id)| node(control_id, frame(index as f32)))
        .collect::<Vec<_>>();
    let mut optimized = retired.clone();

    retired_apply_compact_details_preview_layout(&mut retired, 20.0, 30.0, 320.0, 96.0);
    apply_compact_details_preview_layout(&mut optimized, 20.0, 30.0, 320.0, 96.0);

    assert_frame_outputs_eq(&optimized, &retired);
}

#[test]
#[ignore = "release performance gate; run through the managed Editor57 validator"]
fn editor57_compact_parent_single_pass_details_preview_release_benchmark() {
    let source = (0..BENCHMARK_NODES)
        .map(|index| {
            node(
                DETAILS_PREVIEW_CONTROL_IDS[index % DETAILS_PREVIEW_CONTROL_IDS.len()],
                ViewTemplateFrameData::default(),
            )
        })
        .collect::<Vec<_>>();
    let mut retired = source.clone();
    let mut optimized = source.clone();
    retired_apply_compact_details_preview_layout(&mut retired, 20.0, 30.0, 320.0, 96.0);
    apply_compact_details_preview_layout(&mut optimized, 20.0, 30.0, 320.0, 96.0);
    assert_frame_outputs_eq(&optimized, &retired);

    let mut retired_samples = Vec::with_capacity(BENCHMARK_SAMPLES);
    let mut optimized_samples = Vec::with_capacity(BENCHMARK_SAMPLES);
    for sample in 0..BENCHMARK_SAMPLES {
        if sample % 2 == 0 {
            retired_samples.push(measure_retired_details_preview(&source));
            optimized_samples.push(measure_single_pass_details_preview(&source));
        } else {
            optimized_samples.push(measure_single_pass_details_preview(&source));
            retired_samples.push(measure_retired_details_preview(&source));
        }
    }

    let retired_p95 = nearest_rank(&retired_samples, 95);
    let optimized_p95 = nearest_rank(&optimized_samples, 95);
    let reduction_basis_points = reduction_basis_points(retired_p95, optimized_p95);
    println!(
        "EDITOR57_SINGLE_PASS_COMPACT_DETAILS_PREVIEW_BENCH_V1 samples={} sample_order=alternating percentile_method=nearest_rank node_count={} iterations={} retired_full_node_passes=9 optimized_full_node_passes=1 retired_p95_ns={} optimized_p95_ns={} reduction_basis_points={} retired_ns={} optimized_ns={}",
        BENCHMARK_SAMPLES,
        BENCHMARK_NODES,
        BENCHMARK_ITERATIONS,
        retired_p95,
        optimized_p95,
        reduction_basis_points,
        join_samples(&retired_samples),
        join_samples(&optimized_samples),
    );
    assert!(
        optimized_p95.saturating_mul(100) <= retired_p95.saturating_mul(35),
        "single-pass details preview P95 must be at most 35% of retired: retired={retired_p95}ns optimized={optimized_p95}ns"
    );
}

fn anchor_benchmark_nodes() -> Vec<ViewTemplateNodeData> {
    let mut nodes = Vec::with_capacity(BENCHMARK_NODES);
    while nodes.len() + 6 < BENCHMARK_NODES {
        nodes.push(node("UnrelatedNode", ViewTemplateFrameData::default()));
    }
    for (index, control_id) in [
        "AssetBrowserRoot",
        "AssetBrowserMainPanel",
        "AssetBrowserUtilityPanel",
        "AssetBrowserSourcesPanel",
        "AssetBrowserContentPanel",
        "AssetBrowserDetailsPanel",
    ]
    .into_iter()
    .enumerate()
    {
        nodes.push(node(control_id, frame(index as f32 + 1.0)));
    }
    nodes
}

fn assert_anchor_outputs_eq(nodes: &[ViewTemplateNodeData]) {
    let retired = retired_anchor_summary(nodes);
    let optimized = compact_layout_anchors(nodes);
    assert_eq!(optimized.root, retired.0);
    assert_eq!(optimized.main, retired.1);
    assert_eq!(optimized.utility, retired.2);
    assert_eq!(optimized.sources, retired.3);
    assert_eq!(optimized.content, retired.4);
    assert_eq!(optimized.details, retired.5);
    assert_eq!(optimized.sources.is_some(), retired.6);
    assert_eq!(optimized.details.is_some(), retired.7);
}

fn retired_anchor_summary(
    nodes: &[ViewTemplateNodeData],
) -> (
    Option<ViewTemplateFrameData>,
    Option<ViewTemplateFrameData>,
    Option<ViewTemplateFrameData>,
    Option<ViewTemplateFrameData>,
    Option<ViewTemplateFrameData>,
    Option<ViewTemplateFrameData>,
    bool,
    bool,
) {
    let root = node_frame(nodes, "AssetBrowserRoot");
    let main = node_frame(nodes, "AssetBrowserMainPanel");
    let utility = node_frame(nodes, "AssetBrowserUtilityPanel");
    let sources = node_frame(nodes, "AssetBrowserSourcesPanel");
    let details = node_frame(nodes, "AssetBrowserDetailsPanel");
    let sources_present = black_box(node_frame(
        black_box(nodes),
        black_box("AssetBrowserSourcesPanel"),
    ))
    .is_some();
    let content = node_frame(nodes, "AssetBrowserContentPanel");
    let details_present = black_box(node_frame(
        black_box(nodes),
        black_box("AssetBrowserDetailsPanel"),
    ))
    .is_some();
    (
        root,
        main,
        utility,
        sources,
        content,
        details,
        sources_present,
        details_present,
    )
}

fn measure_retired_anchor_discovery(nodes: &[ViewTemplateNodeData]) -> u128 {
    let started = Instant::now();
    for _ in 0..BENCHMARK_ITERATIONS {
        let _ = black_box(retired_anchor_summary(black_box(nodes)));
    }
    started.elapsed().as_nanos()
}

fn measure_single_pass_anchor_discovery(nodes: &[ViewTemplateNodeData]) -> u128 {
    let started = Instant::now();
    for _ in 0..BENCHMARK_ITERATIONS {
        let _ = black_box(compact_layout_anchors(black_box(nodes)));
    }
    started.elapsed().as_nanos()
}

fn measure_retired_details_preview(source: &[ViewTemplateNodeData]) -> u128 {
    let mut nodes = source.to_vec();
    let started = Instant::now();
    for _ in 0..BENCHMARK_ITERATIONS {
        retired_apply_compact_details_preview_layout(
            black_box(&mut nodes),
            20.0,
            30.0,
            320.0,
            96.0,
        );
    }
    started.elapsed().as_nanos()
}

fn measure_single_pass_details_preview(source: &[ViewTemplateNodeData]) -> u128 {
    let mut nodes = source.to_vec();
    let started = Instant::now();
    for _ in 0..BENCHMARK_ITERATIONS {
        apply_compact_details_preview_layout(black_box(&mut nodes), 20.0, 30.0, 320.0, 96.0);
    }
    started.elapsed().as_nanos()
}

fn retired_apply_compact_details_preview_layout(
    nodes: &mut [ViewTemplateNodeData],
    x: f32,
    y: f32,
    width: f32,
    preview_height: f32,
) {
    let width = finite_non_negative(width);
    let preview_height = finite_non_negative(preview_height);
    let visual_width = 48.0_f32.min(width * 0.34);
    let text_x = x + visual_width + 14.0;
    let text_width = finite_non_negative(x + width - text_x - 8.0);
    retired_set_node_frame(
        nodes,
        "AssetBrowserDetailsPreviewPanel",
        x,
        y,
        width,
        preview_height,
    );
    retired_set_node_frame(
        nodes,
        "AssetBrowserDetailsPreviewVisualPanel",
        x + 8.0,
        y + 8.0,
        visual_width,
        finite_non_negative(preview_height - 16.0),
    );
    for (control_id, offset_y, line_height) in [
        ("AssetBrowserDetailsPreviewNameText", 10.0, 14.0),
        ("AssetBrowserDetailsPreviewLocatorText", 27.0, 12.0),
        ("AssetBrowserDetailsPreviewKindText", 42.0, 12.0),
        ("AssetBrowserDetailsPreviewIdentityText", 56.0, 12.0),
        ("AssetBrowserDetailsPreviewToolkitText", 69.0, 12.0),
        ("AssetBrowserDetailsPreviewMetaPathText", 82.0, 10.0),
        ("AssetBrowserDetailsPreviewDiagnosticsText", 94.0, 10.0),
    ] {
        retired_set_node_frame(
            nodes,
            control_id,
            text_x,
            y + offset_y,
            text_width,
            compact_line_height(preview_height, offset_y, line_height),
        );
    }
}

fn retired_set_node_frame(
    nodes: &mut [ViewTemplateNodeData],
    control_id: &str,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) {
    for node in nodes
        .iter_mut()
        .filter(|node| node.control_id == control_id)
    {
        node.frame.x = finite_coordinate(x);
        node.frame.y = finite_coordinate(y);
        node.frame.width = finite_non_negative(width);
        node.frame.height = finite_non_negative(height);
    }
}

fn assert_frame_outputs_eq(actual: &[ViewTemplateNodeData], expected: &[ViewTemplateNodeData]) {
    assert_eq!(actual.len(), expected.len());
    for (actual, expected) in actual.iter().zip(expected) {
        assert_eq!(actual.control_id, expected.control_id);
        assert_eq!(actual.frame, expected.frame);
    }
}

fn node(control_id: &str, frame: ViewTemplateFrameData) -> ViewTemplateNodeData {
    ViewTemplateNodeData {
        node_id: control_id.into(),
        control_id: control_id.into(),
        frame,
        ..ViewTemplateNodeData::default()
    }
}

fn frame(seed: f32) -> ViewTemplateFrameData {
    ViewTemplateFrameData {
        x: seed,
        y: seed + 1.0,
        width: seed + 2.0,
        height: seed + 3.0,
    }
}

fn reduction_basis_points(retired_p95: u128, optimized_p95: u128) -> u128 {
    10_000_u128.saturating_sub(
        optimized_p95
            .saturating_mul(10_000)
            .checked_div(retired_p95)
            .unwrap_or(0),
    )
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

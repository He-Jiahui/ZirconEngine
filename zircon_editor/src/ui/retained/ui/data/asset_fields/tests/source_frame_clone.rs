use std::hint::black_box;
use std::time::Instant;

use super::{host_contract, source_section_frame};

const SAMPLE_PAIRS: usize = 21;
const CLONES_PER_SAMPLE: usize = 65_536;

#[test]
fn optimization_batch_20260826ef_editor121_frame_projection_preserves_coordinates() {
    let node = fixture_node();
    let frame = source_section_frame(&node);

    assert_eq!(frame.x, 13.0);
    assert_eq!(frame.y, 21.0);
    assert_eq!(frame.width, 377.0);
    assert_eq!(frame.height, 144.0);
    assert_eq!(node.text.as_str(), "detail field payload");
}

#[test]
fn optimization_batch_20260826ef_editor121_frame_projection_clones_only_frame() {
    let source = include_str!("../section_nodes.rs");
    let append_start = source
        .find("pub(super) fn append_detail_section_nodes")
        .unwrap();
    let append_end = source[append_start..]
        .find("fn ui_asset_detail_binding_id")
        .map(|offset| append_start + offset)
        .unwrap();
    let append_source = &source[append_start..append_end];
    assert!(!append_source.contains("nodes[section_index].clone()"));
    assert_eq!(
        append_source
            .matches("source_section_frame(&nodes[section_index])")
            .count(),
        2
    );

    let helper_start = source.find("fn source_section_frame").unwrap();
    let helper_end = source[helper_start..]
        .find("pub(super) fn append_detail_section_nodes")
        .map(|offset| helper_start + offset)
        .unwrap();
    let helper_source = &source[helper_start..helper_end];
    assert!(helper_source.contains("node.frame.clone()"));
    assert!(!helper_source.contains("node.clone()"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826ef_editor121_ui_asset_detail_frame_only_clone_bench() {
    let node = fixture_node();
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure_legacy(&node));
            optimized_samples.push(measure_optimized(&node));
        } else {
            optimized_samples.push(measure_optimized(&node));
            legacy_samples.push(measure_legacy(&node));
        }
    }
    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "EDITOR121_UI_ASSET_DETAIL_FRAME_ONLY_CLONE_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
clones_per_sample={CLONES_PER_SAMPLE} legacy_node_fields=186 optimized_frame_fields=4 \
legacy_clones_per_projection=1 optimized_node_clones_per_projection=0 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70),
        "frame-only clone P95 {optimized_p95_ns}ns must be at most 70% of full-node clone P95 {legacy_p95_ns}ns"
    );
}

fn fixture_node() -> host_contract::TemplatePaneNodeData {
    host_contract::TemplatePaneNodeData {
        node_id: "detail-node".into(),
        parent_node_id: "detail-parent".into(),
        control_id: "InspectorSection".into(),
        role: "InputField".into(),
        text: "detail field payload".into(),
        label_text: "Inspector label".into(),
        component_role: "input-field".into(),
        component_category: "editor-detail".into(),
        value_text: "A representative retained value".into(),
        validation_message: "A representative validation payload".into(),
        action_id: "ui_asset_detail|asset|field".into(),
        edit_action_id: "ui_asset_detail_draft|asset|field".into(),
        commit_action_id: "ui_asset_detail|asset|field|commit".into(),
        frame: host_contract::TemplateNodeFrameData {
            x: 13.0,
            y: 21.0,
            width: 377.0,
            height: 144.0,
        },
        ..host_contract::TemplatePaneNodeData::default()
    }
}

#[inline(never)]
fn legacy_source_section_frame(
    node: &host_contract::TemplatePaneNodeData,
) -> host_contract::TemplateNodeFrameData {
    let cloned = black_box(node.clone());
    let frame = cloned.frame.clone();
    black_box(cloned);
    frame
}

fn measure_legacy(node: &host_contract::TemplatePaneNodeData) -> u128 {
    let started = Instant::now();
    let mut checksum = 0.0f32;
    for _ in 0..CLONES_PER_SAMPLE {
        let frame = black_box(legacy_source_section_frame(black_box(node)));
        checksum += frame.x + frame.height;
    }
    black_box(checksum);
    started.elapsed().as_nanos().max(1)
}

fn measure_optimized(node: &host_contract::TemplatePaneNodeData) -> u128 {
    let started = Instant::now();
    let mut checksum = 0.0f32;
    for _ in 0..CLONES_PER_SAMPLE {
        let frame = black_box(source_section_frame(black_box(node)));
        checksum += frame.x + frame.height;
    }
    black_box(checksum);
    started.elapsed().as_nanos().max(1)
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

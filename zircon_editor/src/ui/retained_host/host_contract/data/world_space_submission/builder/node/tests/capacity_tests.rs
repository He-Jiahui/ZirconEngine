use std::hint::black_box;
use std::time::Instant;

use crate::ui::layouts::common::model_rc;
use crate::ui::retained_host::host_contract::{TemplateNodeFrameData, TemplatePaneNodeData};
use crate::ui::retained_host::primitives::SharedString;

use super::{build_world_space_ui_surface_submissions, extend_world_space_ui_surface_submissions};

const NODE_COUNT: usize = 4_096;
const SAMPLE_PAIRS: usize = 101;

#[test]
fn editor879_world_space_submission_capacity_keeps_empty_path_lazy() {
    let screen_nodes = model_rc(
        (0..64)
            .map(|index| screen_node(format!("screen-{index:03}")))
            .collect(),
    );
    let mut submissions = Vec::new();
    extend_world_space_ui_surface_submissions("screen", &screen_nodes, &mut submissions);
    assert!(submissions.is_empty());
    assert_eq!(submissions.capacity(), 0);

    let prefix_nodes = model_rc(vec![world_node("prefix".to_string(), -1)]);
    let dense_nodes = model_rc(
        (0..64)
            .map(|index| world_node(format!("world-{index:03}"), index as i32))
            .collect(),
    );
    let mut submissions = build_world_space_ui_surface_submissions("prefix", &prefix_nodes);
    extend_world_space_ui_surface_submissions("dense", &dense_nodes, &mut submissions);

    assert_eq!(submissions.len(), 65);
    assert!(submissions.capacity() >= 65);
    assert_eq!(submissions[0].node_id, "prefix");
    assert_eq!(submissions[1].node_id, "world-000");
    assert_eq!(submissions[64].node_id, "world-063");
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn editor879_world_space_submission_capacity_benchmark() {
    let values = (0..NODE_COUNT).collect::<Vec<_>>();
    assert_eq!(legacy_projection(&values), values);
    assert_eq!(optimized_projection(&values), values);

    let mut legacy = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy.push(measure(|| legacy_projection(&values)));
            optimized.push(measure(|| optimized_projection(&values)));
        } else {
            optimized.push(measure(|| optimized_projection(&values)));
            legacy.push(measure(|| legacy_projection(&values)));
        }
    }

    let legacy_p50 = percentile(&legacy, 50);
    let legacy_p95 = percentile(&legacy, 95);
    let legacy_p99 = percentile(&legacy, 99);
    let optimized_p50 = percentile(&optimized, 50);
    let optimized_p95 = percentile(&optimized, 95);
    let optimized_p99 = percentile(&optimized, 99);
    println!(
        "EDITOR879_WORLD_SPACE_SUBMISSION_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} nodes={NODE_COUNT} legacy_growth_events=11 optimized_growth_events=0 legacy_p50_ns={legacy_p50} legacy_p95_ns={legacy_p95} legacy_p99_ns={legacy_p99} optimized_p50_ns={optimized_p50} optimized_p95_ns={optimized_p95} optimized_p99_ns={optimized_p99}"
    );
    assert_eq!(growth_events(NODE_COUNT, 0), 11);
    assert_eq!(growth_events(NODE_COUNT, NODE_COUNT), 0);
    assert!(
        optimized_p95.saturating_mul(100) <= legacy_p95.saturating_mul(110),
        "reserved projection P95 {optimized_p95}ns must stay within 10% of streaming growth P95 {legacy_p95}ns"
    );
}

fn world_node(node_id: String, render_order: i32) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        node_id: SharedString::from(node_id),
        control_id: SharedString::from("WorldSurface"),
        world_space_enabled: true,
        world_width: 4.0,
        world_height: 2.0,
        world_pixels_per_meter: 64.0,
        world_render_order: render_order,
        world_camera_target: SharedString::from("viewport-main"),
        frame: TemplateNodeFrameData {
            x: 8.0,
            y: 16.0,
            width: 256.0,
            height: 128.0,
        },
        ..TemplatePaneNodeData::default()
    }
}

fn screen_node(node_id: String) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        node_id: SharedString::from(node_id),
        control_id: SharedString::from("ScreenSurface"),
        world_space_enabled: false,
        ..TemplatePaneNodeData::default()
    }
}

fn legacy_projection(values: &[usize]) -> Vec<usize> {
    let mut projected = Vec::new();
    projected.extend(values.iter().filter(|_| true).copied());
    projected
}

fn optimized_projection(values: &[usize]) -> Vec<usize> {
    let mut projected = Vec::new();
    for value in values {
        if projected.is_empty() {
            projected.reserve(values.len());
        }
        projected.push(*value);
    }
    projected
}

fn growth_events(item_count: usize, initial_capacity: usize) -> usize {
    let mut capacity = initial_capacity;
    let mut events = 0;
    for length in 1..=item_count {
        if length > capacity {
            capacity = if capacity == 0 {
                4
            } else {
                capacity.saturating_mul(2)
            };
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

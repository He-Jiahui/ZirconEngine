use std::hint::black_box;
use std::time::{Duration, Instant};

use crate::ui::layouts::common::model_rc;
use crate::ui::retained_host::host_contract::{TemplateNodeFrameData, TemplatePaneNodeData};
use crate::ui::retained_host::primitives::SharedString;

use super::{build_world_space_ui_surface_submissions, extend_world_space_ui_surface_submissions};

const PERFORMANCE_MARKER: &str = "EDITOR84_WORLD_SPACE_SUBMISSION_DIRECT_APPEND_BENCH_V1";

#[test]
fn optimization_batch_20260826cu_editor84_direct_append_preserves_existing_submissions() {
    let initial_nodes = model_rc(vec![world_node("existing", 0)]);
    let appended_nodes = model_rc(vec![
        world_node("second", 2),
        screen_node("screen-only"),
        world_node("first", 1),
    ]);
    let mut submissions = build_world_space_ui_surface_submissions("initial", &initial_nodes);

    extend_world_space_ui_surface_submissions("appended", &appended_nodes, &mut submissions);

    assert_eq!(submissions.len(), 3);
    assert_eq!(submissions[0].node_id, "existing");
    assert_eq!(submissions[1].node_id, "second");
    assert_eq!(submissions[2].node_id, "first");
    assert_eq!(submissions[1].surface_id, "appended");
}

#[test]
fn optimization_batch_20260826cu_editor84_standalone_builder_keeps_sorted_contract() {
    let nodes = model_rc(vec![
        world_node("late", 20),
        world_node("same-b", 4),
        world_node("same-a", 4),
        world_node("early", -3),
    ]);

    let submissions = build_world_space_ui_surface_submissions("surface", &nodes);
    let node_ids = submissions
        .iter()
        .map(|submission| submission.node_id.as_str())
        .collect::<Vec<_>>();

    assert_eq!(node_ids, ["early", "same-a", "same-b", "late"]);
}

#[test]
#[ignore = "release-only world-space submission append performance gate"]
fn optimization_batch_20260826cu_editor84_direct_append_performance_evidence() {
    const GROUP_COUNT: usize = 32;
    const NODES_PER_GROUP: usize = 96;
    const SAMPLE_COUNT: usize = 15;

    assert_eq!(
        PERFORMANCE_MARKER,
        "EDITOR84_WORLD_SPACE_SUBMISSION_DIRECT_APPEND_BENCH_V1"
    );
    let groups = (0..GROUP_COUNT)
        .map(|group| {
            model_rc(
                (0..NODES_PER_GROUP)
                    .map(|node| {
                        world_node_owned(
                            format!("world-node-{group:03}-{node:04}"),
                            ((node * 37 + group * 13) % NODES_PER_GROUP) as i32,
                        )
                    })
                    .collect(),
            )
        })
        .collect::<Vec<_>>();

    for _ in 0..3 {
        black_box(legacy_collect_groups(&groups));
        black_box(direct_collect_groups(&groups));
    }

    let mut legacy_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_COUNT);
    for sample in 0..SAMPLE_COUNT {
        if sample % 2 == 0 {
            legacy_samples.push(measure(|| black_box(legacy_collect_groups(&groups))));
            optimized_samples.push(measure(|| black_box(direct_collect_groups(&groups))));
        } else {
            optimized_samples.push(measure(|| black_box(direct_collect_groups(&groups))));
            legacy_samples.push(measure(|| black_box(legacy_collect_groups(&groups))));
        }
    }

    let legacy_p50_ns = percentile_ns(&mut legacy_samples, 50);
    let legacy_p95_ns = percentile_ns(&mut legacy_samples, 95);
    let optimized_p50_ns = percentile_ns(&mut optimized_samples, 50);
    let optimized_p95_ns = percentile_ns(&mut optimized_samples, 95);
    println!(
        "{PERFORMANCE_MARKER} legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} groups={GROUP_COUNT} nodes_per_group={NODES_PER_GROUP} samples={SAMPLE_COUNT} legacy_temporary_vectors={GROUP_COUNT} optimized_temporary_vectors=0 legacy_local_sorts={GROUP_COUNT} optimized_local_sorts=0"
    );
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70),
        "direct append P95 {optimized_p95_ns}ns must be at most 70% of temporary-vector P95 {legacy_p95_ns}ns"
    );
}

fn legacy_collect_groups(
    groups: &[crate::ui::retained_host::primitives::ModelRc<TemplatePaneNodeData>],
) -> Vec<super::WorldSpaceUiSurfaceSubmission> {
    let mut submissions = Vec::new();
    for nodes in groups {
        submissions.extend(build_world_space_ui_surface_submissions(
            "benchmark-surface",
            nodes,
        ));
    }
    submissions
}

fn direct_collect_groups(
    groups: &[crate::ui::retained_host::primitives::ModelRc<TemplatePaneNodeData>],
) -> Vec<super::WorldSpaceUiSurfaceSubmission> {
    let mut submissions = Vec::new();
    for nodes in groups {
        extend_world_space_ui_surface_submissions("benchmark-surface", nodes, &mut submissions);
    }
    submissions
}

fn world_node(node_id: &'static str, render_order: i32) -> TemplatePaneNodeData {
    world_node_owned(node_id.to_string(), render_order)
}

fn world_node_owned(node_id: String, render_order: i32) -> TemplatePaneNodeData {
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

fn screen_node(node_id: &'static str) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        node_id: SharedString::from(node_id),
        control_id: SharedString::from("ScreenSurface"),
        world_space_enabled: false,
        ..TemplatePaneNodeData::default()
    }
}

fn measure<T>(run: impl FnOnce() -> T) -> Duration {
    let started = Instant::now();
    black_box(run());
    started.elapsed()
}

fn percentile_ns(samples: &mut [Duration], percentile: usize) -> u128 {
    samples.sort_unstable();
    let rank = (samples.len() * percentile).div_ceil(100);
    samples[rank.saturating_sub(1)].as_nanos()
}

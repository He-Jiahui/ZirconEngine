use std::hint::black_box;
use std::time::Instant;

use super::*;

#[test]
fn optimization_batch_hm_editor589_pane_projection_moves_owned_view_data() {
    let conversion = include_str!("../../apply_presentation.rs");
    let pane = include_str!("../pane_conversion.rs");
    let toolbar = include_str!("../../../app/viewport/toolbar_pointer/chrome_projection.rs");

    let viewport = conversion
        .split("pub(crate) fn to_host_contract_scene_viewport_chrome")
        .nth(1)
        .expect("viewport conversion")
        .split("fn to_host_contract_animation_editor_pane")
        .next()
        .expect("viewport conversion end");
    assert!(viewport.contains("data: view_data::SceneViewportChromeData"));
    assert!(!viewport.contains("data.mode.clone()"));
    assert!(pane.contains("to_host_contract_scene_viewport_chrome(data.viewport)"));
    assert!(pane.contains("project_overview)"));
    assert!(!pane.contains("project_overview.clone()"));
    assert!(!toolbar.contains("to_host_contract_scene_viewport_chrome(&"));
}

fn viewport_fixture(index: usize) -> view_data::SceneViewportChromeData {
    let text = |field: &str| {
        format!(
            "editor589-{field}-{index:05}-{}",
            "viewport-shared-string".repeat(3)
        )
        .into()
    };
    view_data::SceneViewportChromeData {
        mode: text("mode"),
        transform_space: text("transform-space"),
        pivot_mode: text("pivot-mode"),
        projection_mode: text("projection-mode"),
        view_orientation: text("view-orientation"),
        display_mode: text("display-mode"),
        grid_mode: text("grid-mode"),
        gizmos_enabled: true,
        preview_lighting: true,
        preview_skybox: false,
        translate_snap: 0.5,
        rotate_snap_deg: 15.0,
        scale_snap: 0.1,
        translate_snap_label: text("translate-label"),
        rotate_snap_label: text("rotate-label"),
        scale_snap_label: text("scale-label"),
    }
}

fn legacy_viewport_projection(
    data: &view_data::SceneViewportChromeData,
) -> host_contract::SceneViewportChromeData {
    host_contract::SceneViewportChromeData {
        mode: data.mode.clone(),
        transform_space: data.transform_space.clone(),
        pivot_mode: data.pivot_mode.clone(),
        projection_mode: data.projection_mode.clone(),
        view_orientation: data.view_orientation.clone(),
        display_mode: data.display_mode.clone(),
        grid_mode: data.grid_mode.clone(),
        gizmos_enabled: data.gizmos_enabled,
        preview_lighting: data.preview_lighting,
        preview_skybox: data.preview_skybox,
        translate_snap: data.translate_snap,
        rotate_snap_deg: data.rotate_snap_deg,
        scale_snap: data.scale_snap,
        translate_snap_label: data.translate_snap_label.clone(),
        rotate_snap_label: data.rotate_snap_label.clone(),
        scale_snap_label: data.scale_snap_label.clone(),
        toolbar_surface_frame: None,
        toolbar_template_nodes: Default::default(),
        toolbar_surface_key: Default::default(),
        toolbar_enter_play_enabled: false,
        toolbar_exit_play_enabled: false,
        toolbar_is_playing: false,
    }
}

fn measure_legacy_viewports(inputs: &[view_data::SceneViewportChromeData]) -> u128 {
    let started = Instant::now();
    let outputs = inputs
        .iter()
        .map(legacy_viewport_projection)
        .collect::<Vec<_>>();
    black_box(&outputs);
    let elapsed = started.elapsed().as_nanos().max(1);
    drop(outputs);
    elapsed
}

fn measure_moved_viewports(mut inputs: Vec<view_data::SceneViewportChromeData>) -> u128 {
    let started = Instant::now();
    let outputs = inputs
        .drain(..)
        .map(to_host_contract_scene_viewport_chrome)
        .collect::<Vec<_>>();
    black_box(&outputs);
    let elapsed = started.elapsed().as_nanos().max(1);
    drop(outputs);
    elapsed
}

fn percentile_95(samples: &mut [u128]) -> u128 {
    samples.sort_unstable();
    samples[(samples.len() - 1) * 95 / 100]
}

#[test]
#[ignore = "release performance evidence"]
fn optimization_batch_hm_editor589_viewport_move_performance_evidence() {
    const SAMPLE_PAIRS: usize = 21;
    const VIEWPORTS_PER_SAMPLE: usize = 32_768;

    let seed = (0..VIEWPORTS_PER_SAMPLE)
        .map(viewport_fixture)
        .collect::<Vec<_>>();
    let expected = legacy_viewport_projection(&seed[0]);
    let optimized = to_host_contract_scene_viewport_chrome(seed[0].clone());
    assert_eq!(optimized.mode, expected.mode);
    assert_eq!(optimized.scale_snap_label, expected.scale_snap_label);

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        let legacy_input = seed.clone();
        let optimized_input = seed.clone();
        if pair % 2 == 0 {
            legacy_samples.push(measure_legacy_viewports(&legacy_input));
            optimized_samples.push(measure_moved_viewports(optimized_input));
        } else {
            optimized_samples.push(measure_moved_viewports(optimized_input));
            legacy_samples.push(measure_legacy_viewports(&legacy_input));
        }
    }

    let legacy_p95 = percentile_95(&mut legacy_samples);
    let optimized_p95 = percentile_95(&mut optimized_samples);
    println!(
        "EDITOR589_VIEWPORT_OWNERSHIP_MOVE_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
         viewports_per_sample={VIEWPORTS_PER_SAMPLE} legacy_shared_string_clones_per_viewport=10 \
         optimized_shared_string_clones_per_viewport=0 legacy_p95_ns={legacy_p95} \
         optimized_p95_ns={optimized_p95}"
    );
    assert!(
        optimized_p95 * 100 <= legacy_p95 * 70,
        "moved viewport P95 {optimized_p95}ns exceeded 70% of cloned P95 {legacy_p95}ns"
    );
}

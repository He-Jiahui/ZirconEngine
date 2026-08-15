use std::cell::Cell;

use super::{
    collect_irradiance_sample_positions, ensure_compiled_scene_graph_resources, RenderGenerationIds,
};
use crate::graphics::scene::scene_renderer::SceneRendererDeferredLightingProfile;
use crate::graphics::types::GraphicsError;

#[test]
fn environment_only_profile_is_rejected_before_compiled_graph_execution() {
    assert!(ensure_compiled_scene_graph_resources(
        SceneRendererDeferredLightingProfile::FullScene,
        true,
        true,
    )
    .is_ok());
    assert!(ensure_compiled_scene_graph_resources(
        SceneRendererDeferredLightingProfile::StandardPbrPreview,
        true,
        true,
    )
    .is_ok());
    let error = ensure_compiled_scene_graph_resources(
        SceneRendererDeferredLightingProfile::EnvironmentOnlyPbrPreview,
        true,
        true,
    )
    .expect_err("expected output-transfer-only rejection");
    assert!(matches!(error, GraphicsError::Asset(_)));
    assert!(error
        .to_string()
        .contains("cannot execute a compiled scene graph"));
    let error = ensure_compiled_scene_graph_resources(
        SceneRendererDeferredLightingProfile::FullScene,
        false,
        true,
    )
    .expect_err("expected missing full post-process rejection");
    assert!(matches!(error, GraphicsError::Asset(_)));
    assert!(error.to_string().contains("full post-process resources"));
    let error = ensure_compiled_scene_graph_resources(
        SceneRendererDeferredLightingProfile::FullScene,
        true,
        false,
    )
    .expect_err("expected missing scene-clear rejection");
    assert!(matches!(error, GraphicsError::Asset(_)));
    assert!(error.to_string().contains("scene-clear resources"));
}

#[test]
fn gpu_timer_frame_generation_stays_independent_from_mesh_command_cache_generation() {
    let generation_ids = RenderGenerationIds::new(41, 7);

    assert_eq!(generation_ids.timer_frame(), 41);
    assert_eq!(generation_ids.mesh_commands, 7);
}

#[test]
fn frame_without_irradiance_volumes_does_not_scan_mesh_positions() {
    let visited = Cell::new(0);
    let positions = (0..4).inspect(|_| visited.set(visited.get() + 1));

    let collected = collect_irradiance_sample_positions(false, positions);

    assert!(collected.is_none());
    assert_eq!(visited.get(), 0);
}

#[test]
fn frame_with_irradiance_volumes_collects_mesh_positions_once() {
    let visited = Cell::new(0);
    let positions = (0..4).inspect(|_| visited.set(visited.get() + 1));

    let collected = collect_irradiance_sample_positions(true, positions);

    assert_eq!(collected, Some(vec![0, 1, 2, 3]));
    assert_eq!(visited.get(), 4);
}

#[test]
fn runtime_prepare_starts_after_shared_readback_and_gpu_timer_admission() {
    let source = include_str!("render.rs");
    let runtime_prepare_prefix = [
        "let mut advanced_plugin_readbacks = match ",
        "self.execute_runtime_prepare_passes",
    ]
    .concat();
    let readback_admission = source
        .find("let readback_ready = self\n            .readback_queue\n            .prepare_frame")
        .expect("compiled scene rendering must reserve the shared readback frame");
    let timer_begin = source
        .find("timer.begin_frame(generation_ids.timer_frame());")
        .expect("compiled scene rendering must begin the shared GPU timer");
    let runtime_prepare = source
        .find(&runtime_prepare_prefix)
        .expect("compiled scene rendering must execute runtime prepare");

    assert!(readback_admission < timer_begin);
    assert!(timer_begin < runtime_prepare);
}

#[test]
fn runtime_prepare_profiles_join_the_frame_profile_before_graph_execution() {
    let source = include_str!("render.rs");
    let profile_handoff = ["advanced_plugin_readbacks.", "take_gpu_pass_profiles()"].concat();
    let graph_execution = [
        "let mut graph_execution = ",
        "RenderGraphStageExecution::new(",
    ]
    .concat();
    let profile_handoff = source
        .find(&profile_handoff)
        .expect("runtime-prepare profiles must be recorded");
    let graph_execution = source
        .find(&graph_execution)
        .expect("compiled graph execution must remain explicit");

    assert!(profile_handoff < graph_execution);
}

#[test]
fn graph_execution_failure_defers_the_shared_gpu_timer_frame() {
    let source = include_str!("render.rs");
    let graph_failure = ["if let Err(error) = ", "graph_execution_result {"].concat();
    let timer_defer = [
        "let _ = timer.",
        "defer_frame(generation_ids.timer_frame());",
    ]
    .concat();
    let graph_failure = source
        .find(&graph_failure)
        .expect("compiled graph execution must retain its failure branch");
    let hzb_readback = source[graph_failure..]
        .find("let hzb_readback_requested")
        .map(|offset| graph_failure + offset)
        .expect("graph failure branch must end before HZB readback handling");

    assert!(source[graph_failure..hzb_readback].contains(&timer_defer));
}

#[test]
fn hzb_cpu_diagnostics_request_readback_only_when_explicitly_enabled() {
    let source = include_str!("render.rs");
    let request_gate = [
        "let hzb_readback_requested = hzb_diagnostics_readback_enabled\n",
        "            && graph_execution_record",
    ]
    .concat();
    let gate = source
        .find(&request_gate)
        .expect("HZB CPU diagnostics must retain an explicit opt-in gate");
    let request = source[gate..]
        .find("culler.request_frame_readbacks(")
        .map(|offset| gate + offset)
        .expect("enabled HZB diagnostics must use the shared readback queue");

    assert!(gate < request);
    let skipped = source[request..]
        .find("} else if hzb_readback_requested {")
        .map(|offset| request + offset)
        .expect("explicit HZB sampling must report a shared-ring admission drop");
    assert!(request < skipped);
    assert!(source[skipped..].contains("culler.record_skipped_readback();"));
}

#[test]
fn empty_taa_reactive_mask_stream_binds_the_shared_black_texture() {
    let source = include_str!("bind_taa_reactive_mask_graph_resource.rs");

    assert!(source.contains("PostProcessGraphResourceNames::TAA_REACTIVE_MASK"));
    assert!(source.contains("taa_reactive_mask_stream().is_empty()"));
    assert!(source.contains("post_process.black_texture_view()"));
    assert!(source.contains("import_borrowed_texture_view"));
}

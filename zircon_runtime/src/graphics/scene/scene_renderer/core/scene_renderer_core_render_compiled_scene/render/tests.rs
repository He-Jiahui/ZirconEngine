use std::cell::Cell;

use super::super::frame_lifecycle::{RenderGenerationIds, ensure_compiled_scene_graph_resources};
use super::super::irradiance_volume_selection::collect_irradiance_sample_positions;
use crate::graphics::scene::scene_renderer::SceneRendererDeferredLightingProfile;
use crate::graphics::types::GraphicsError;

fn compiled_scene_execution_source() -> String {
    [
        include_str!("compiled_scene_frame_foundation.rs"),
        include_str!("prepare_compiled_scene_mesh_submission.rs"),
        include_str!("prepare_compiled_scene_graph_frame.rs"),
        include_str!("render.rs"),
        include_str!("commit_compiled_scene_frame_success.rs"),
    ]
    .join("\n")
}

#[test]
fn compiled_scene_core_does_not_own_the_frame_completion_pump() {
    let source = include_str!("render.rs");

    assert!(!source.contains("poll_submission_completions"));
    assert!(source.contains("let scene_submission ="));
}

#[test]
fn light_cookie_profile_resets_at_frame_entry_and_emits_only_after_scene_validation() {
    let source = compiled_scene_execution_source();
    let reset = source
        .find("self.mesh_pipelines.light_cookies.begin_profile_frame()")
        .expect("light cookie profile frame reset");
    let graph_execution = source
        .find("self.execute_compiled_scene_graph_stages(")
        .expect("compiled graph execution");
    let scene_validation = source
        .find("submission_transaction.validate_scene_submission(scene_submission)")
        .expect("compiled scene transaction validation");
    let emit = source
        .find("self.mesh_pipelines.light_cookies.emit_profile_frame()")
        .expect("light cookie successful-frame profile emission");

    assert!(reset < graph_execution);
    assert!(graph_execution < scene_validation);
    assert!(scene_validation < emit);
}

#[test]
fn runtime_prepare_state_commits_only_after_scene_submission_validation() {
    let source = compiled_scene_execution_source();
    let scene_validation = source
        .find("submission_transaction.validate_scene_submission(scene_submission)")
        .expect("compiled scene transaction validation");
    let runtime_prepare_commit = source
        .find("advanced_plugin_readbacks.commit_runtime_prepare_frame_transactions()")
        .expect("runtime prepare state transaction commit");
    let outputs = source
        .find("advanced_plugin_readbacks.into_outputs()")
        .expect("runtime prepare renderer outputs extraction");

    assert!(scene_validation < runtime_prepare_commit);
    assert!(runtime_prepare_commit < outputs);
}

#[test]
fn product_frame_timestamp_fact_does_not_require_raw_queue_authority() {
    let compiled = include_str!("render.rs");
    let direct = include_str!("../../scene_renderer_core_render_scene/render_scene.rs");
    let submission = include_str!("submit_compiled_scene_frame.rs");

    for source in [compiled, direct] {
        let production = source.split("\n#[cfg(test)]").next().unwrap_or_default();
        assert!(!production.contains("let queue = &backend.queue;"));
        assert!(!production.contains("queue.get_timestamp_period()"));
        assert!(production.contains("backend.render_device.timestamp_period_ns()"));
    }
    assert!(submission.contains("#[cfg(test)]\n    pub(super) queue: &'a wgpu::Queue"));
    assert!(!submission.contains("let _ = (device, queue, streamer, frame);"));
}

#[test]
fn compiled_submission_keeps_synchronous_readback_inputs_test_only() {
    let compiled = include_str!("render.rs");
    let submission = include_str!("submit_compiled_scene_frame.rs");

    assert!(submission.contains("#[cfg(test)]\n    pub(super) device: &'a wgpu::Device",));
    assert!(submission.contains("#[cfg(test)]\n    pub(super) streamer: &'a ResourceStreamer",));
    assert!(submission.contains("#[cfg(test)]\n    pub(super) frame: &'a ViewportRenderFrame",));
    assert!(!submission.contains("#[cfg(not(test))]\n        let _ = (device, streamer, frame);",));
    assert!(compiled.contains("#[cfg(test)]\n                device,"));
    assert!(compiled.contains("#[cfg(test)]\n                streamer,"));
    assert!(compiled.contains("#[cfg(test)]\n                frame,"));
}

#[test]
fn environment_only_profile_is_rejected_before_compiled_graph_execution() {
    assert!(
        ensure_compiled_scene_graph_resources(
            SceneRendererDeferredLightingProfile::FullScene,
            true,
            true,
        )
        .is_ok()
    );
    assert!(
        ensure_compiled_scene_graph_resources(
            SceneRendererDeferredLightingProfile::StandardPbrPreview,
            true,
            true,
        )
        .is_ok()
    );
    let error = ensure_compiled_scene_graph_resources(
        SceneRendererDeferredLightingProfile::EnvironmentOnlyPbrPreview,
        true,
        true,
    )
    .expect_err("expected output-transfer-only rejection");
    assert!(matches!(error, GraphicsError::Asset(_)));
    assert!(
        error
            .to_string()
            .contains("cannot execute a compiled scene graph")
    );
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
fn compiled_material_feature_census_covers_every_graph_context() {
    let source = include_str!("compiled_scene_frame_foundation.rs");
    let feature_gate = source
        .split("let material_pipeline_features = MaterialPipelineFeatureSet::from_executor_ids(")
        .nth(1)
        .and_then(|source| {
            source
                .split("Ok(PreparedCompiledSceneFrameFoundation")
                .next()
        })
        .expect("compiled material pipeline feature gate");

    assert!(feature_gate.contains("pipeline.graph()"));
    assert!(!feature_gate.contains(".has_material_pipeline_admission_work()"));
    assert!(!feature_gate.contains(".has_active_staged_material_candidates()"));
}

#[test]
fn runtime_prepare_starts_after_native_query_scope_and_timer_attachment() {
    let source = compiled_scene_execution_source();
    let runtime_prepare_prefix = [
        "let mut advanced_plugin_readbacks = match ",
        "self.execute_runtime_prepare_passes",
    ]
    .concat();
    let query_admission = source
        .find(".begin_product_diagnostic_query_scope(")
        .expect("compiled scene rendering must reserve the native query frame");
    let timer_begin = source
        .find("scope.attach_timers(")
        .expect("compiled scene rendering must attach adapters to native query sets");
    let runtime_prepare = source
        .find(&runtime_prepare_prefix)
        .expect("compiled scene rendering must execute runtime prepare");

    assert!(query_admission < timer_begin);
    assert!(timer_begin < runtime_prepare);
}

#[test]
fn static_environment_uploads_share_the_compiled_frame_encoder() {
    let source = compiled_scene_execution_source();
    let encoder = source
        .find("let mut encoder = device.create_command_encoder")
        .expect("compiled scene render owns a frame encoder");
    let write_uniform = source
        .find("self.write_scene_uniform(")
        .expect("compiled scene render must update scene bindings");

    assert!(encoder < write_uniform);
}

#[test]
fn compiled_frame_buffer_upload_is_merged_after_graph_success_and_before_scene_submission() {
    let source = compiled_scene_execution_source();
    let constants = source
        .find("let mut frame_buffer_uploads = self.write_scene_uniform(")
        .expect("compiled rendering must prepare the packed scene constants");
    let exposure = source
        .find(".prepare_exposure_params_upload(")
        .expect("compiled rendering must prepare shared exposure params once per frame");
    let irradiance = source
        .find(".prepare(selected_irradiance_volume, &mut frame_buffer_uploads)")
        .expect("compiled rendering must append irradiance params to the frame batch");
    let shadow = source
        .find("shadow_atlas_prepared_upload.append_to(&mut frame_buffer_uploads)")
        .expect("compiled rendering must append shadow data to the frame batch");
    let gpu_scene = source
        .find("gpu_scene_prepared_upload.append_to(&self.gpu_scene, frame_buffer_uploads)")
        .expect("compiled rendering must merge GPU Scene writes into the frame batch");
    let mesh_indirect = source
        .find("mesh_indirect_prepared_upload.append_to(frame_buffer_uploads)")
        .expect("compiled rendering must merge mesh indirect writes into the frame batch");
    let icon_prepare = source
        .find("let prepared_overlays = match prepare_overlay_buffers(")
        .expect("compiled rendering must prepare viewport icon uploads in the frame batch");
    let graph_success = source
        .find("let mut graph_buffer_uploads = graph_execution.take_buffer_uploads()")
        .expect("compiled rendering must retain uploads only after graph execution succeeds");
    let ui_commit_take = source
        .find("graph_execution.take_screen_space_ui_upload_commits()")
        .expect("compiled rendering must retain UI commit tokens after graph success");
    let graph_merge = source
        .find("frame_buffer_uploads.append(&mut graph_buffer_uploads)")
        .expect("compiled rendering must merge graph pass writes into the frame batch");
    let hzb_commit_take = source
        .find("graph_execution.take_hzb_occlusion_params_commits()")
        .expect("compiled rendering must retain HZB commit tokens after graph success");
    let enqueue = source
        .find(".enqueue_copy_resource_upload_batch(")
        .expect("compiled rendering must accept one frame resource upload batch");
    let ledger = source
        .find("RenderFrameSubmissionProducer::FrameResourceUpload")
        .expect("compiled rendering must retain the merged frame upload ticket");
    let commit = source
        .find("gpu_scene_prepared_upload.commit(&mut self.gpu_scene)")
        .expect("GPU Scene dirty state must commit after backend acceptance");
    let hzb_commit = source
        .find("culler.commit_params_uploads(hzb_occlusion_params_commits)")
        .expect("HZB params state must commit after upload ticket retention");
    let ui_commit = source
        .find("renderer.commit_prepared_upload(prepared)")
        .expect("UI reuse state must commit after upload ticket retention");
    let mesh_indirect_commit = source
        .find(".commit(&mut self.mesh_indirect_draw_workspace)")
        .expect("mesh indirect shadows must commit after upload ticket retention");
    let icon_commit = source
        .find("self.overlay_renderer.commit_pending_icon_uploads()")
        .expect("viewport icon reuse state must commit after upload ticket retention");
    let graph_execution = source
        .find("RenderGraphStageExecution::new(")
        .expect("compiled rendering must execute the graph after frame preparation");
    let scene_submission = source
        .find("self.submit_compiled_scene_frame(")
        .expect("compiled rendering must keep one scene submission owner");
    let scene_validation = source
        .find("submission_transaction.validate_scene_submission(scene_submission)")
        .expect("compiled rendering must validate the scene ticket before resource commit");

    assert!(constants < exposure);
    assert!(exposure < irradiance);
    assert!(irradiance < shadow);
    assert!(shadow < gpu_scene);
    assert!(gpu_scene < mesh_indirect);
    assert!(mesh_indirect < icon_prepare);
    assert!(icon_prepare < graph_execution);
    assert!(graph_execution < graph_success);
    assert!(graph_success < graph_merge);
    assert!(graph_success < ui_commit_take);
    assert!(graph_success < hzb_commit_take);
    assert!(graph_merge < enqueue);
    assert!(enqueue < ledger);
    assert!(ledger < scene_submission);
    assert!(scene_submission < scene_validation);
    assert!(scene_validation < hzb_commit);
    assert!(scene_validation < ui_commit);
    assert!(scene_validation < mesh_indirect_commit);
    assert!(scene_validation < icon_commit);
    assert!(hzb_commit < commit);
    assert!(mesh_indirect_commit < commit);
    assert!(scene_validation < commit);
}

#[test]
fn runtime_prepare_buffer_uploads_join_the_frame_transaction_after_prepare_success() {
    let source = compiled_scene_execution_source();
    let runtime_prepare = source
        .find("self.execute_runtime_prepare_passes(")
        .expect("compiled rendering must execute runtime prepare");
    let take_uploads = source
        .find("advanced_plugin_readbacks.take_runtime_prepare_buffer_uploads()")
        .expect("runtime prepare must hand off its frame-local upload batch");
    let merge_uploads = source
        .find("frame_buffer_uploads.append(&mut runtime_prepare_buffer_uploads)")
        .expect("runtime prepare writes must join the sole frame upload packet");
    let graph_execution = source
        .find("RenderGraphStageExecution::new(")
        .expect("compiled graph execution must begin after prepare succeeds");
    let enqueue = source
        .find(".enqueue_copy_resource_upload_batch(")
        .expect("the merged frame upload packet must have one submission owner");

    assert!(runtime_prepare < take_uploads);
    assert!(take_uploads < merge_uploads);
    assert!(merge_uploads < graph_execution);
    assert!(graph_execution < enqueue);
}

#[test]
fn runtime_prepare_profiles_join_the_frame_profile_before_graph_execution() {
    let source = compiled_scene_execution_source();
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
    let source = compiled_scene_execution_source();
    let graph_failure = ["if let Err(error) = ", "graph_execution_result {"].concat();
    let timer_defer = ["timer.", "defer_frame(generation_ids.timer_frame());"].concat();
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
fn post_begin_graph_failures_release_transient_resources_before_returning() {
    let source = compiled_scene_execution_source();
    let abort = "abort_compiled_scene_graph_resource_frame(";
    let first_pool_begin = source
        .find(".begin_frame(backend.device_profile());")
        .expect("compiled graph resources must begin a transient pool frame");
    let submit = source
        .find("self.submit_compiled_scene_frame(")
        .expect("compiled graph resources must finish through one submit path");
    let post_begin = &source[first_pool_begin..submit];

    assert_eq!(post_begin.matches(abort).count(), 10);
    assert!(post_begin.contains("drop(command_encoders);\n            abort_compiled"));
}

#[test]
fn recorded_realtime_ibl_submission_is_terminalized_on_pre_submit_errors() {
    let source = compiled_scene_execution_source();
    let recorded = source
        .find("let realtime_ibl_submission")
        .expect("realtime IBL submission must remain explicit");
    let submit = source
        .find("self.submit_compiled_scene_frame(")
        .expect("compiled scene must have one submit boundary");
    let pre_submit = &source[recorded..submit];

    assert_eq!(
        pre_submit.matches("abort_realtime_ibl_submission(").count(),
        14
    );

    let terminal = include_str!("terminal_frame_packet.rs");
    assert_eq!(terminal.matches("return Err(error);").count(), 1);
    assert_eq!(
        terminal.matches("defer_gpu_timers(&mut context);").count(),
        1
    );
}

#[test]
fn compiled_output_target_writeback_precedes_query_resolve_and_main_submission() {
    let render = compiled_scene_execution_source();
    let terminal = include_str!("terminal_frame_packet.rs");
    let stages = include_str!("execute_compiled_scene_graph_stages.rs");
    let output_target =
        include_str!("../../../graph_execution/render_pass_execution_context/gpu/output_target.rs");
    let graph_execution = render
        .find("self.execute_compiled_scene_graph_stages(")
        .expect("compiled rendering must execute its authored graph");
    let terminal_prepare = render
        .find("prepare_terminal_frame_packet(")
        .expect("compiled rendering must prepare one terminal frame packet");
    let submit = render
        .find("self.submit_compiled_scene_frame(")
        .expect("compiled rendering must retain one frame submission boundary");

    let query_tail = terminal
        .find(".finish_and_prepare(")
        .expect("compiled typed queries must resolve in the serial tail");
    let viewport_product_copy = terminal
        .find("viewport_product_copy.encode_copy(")
        .expect("viewport product copy must share the compiled scene serial encoder");

    assert!(viewport_product_copy < query_tail);
    assert!(graph_execution < terminal_prepare);
    assert!(terminal_prepare < submit);
    assert!(stages.contains("RenderPassStage::Present"));
    assert!(stages.contains("streamer: Some(streamer)"));
    assert!(output_target.contains("encode_planned_output_target_writeback("));
    assert!(!terminal.contains("encode_output_target_writeback("));
}

#[test]
fn viewport_capture_uses_the_product_diagnostic_tail_and_scene_ticket() {
    let render = compiled_scene_execution_source();
    let terminal = include_str!("terminal_frame_packet.rs");
    let submit = include_str!("submit_compiled_scene_frame.rs");
    let begin = render
        .find(".begin_product_diagnostic_readback_scope(frame_generation)")
        .expect("viewport capture should open a production diagnostic frame");
    let request = render
        .find("backend.enqueue_product_diagnostic_texture_rgba8(")
        .expect("viewport capture should register a bounded native source lease");
    let terminal_prepare = render
        .find("prepare_terminal_frame_packet(")
        .expect("viewport capture should reach the terminal frame packet");
    let prepare = terminal
        .find("scope.prepare(")
        .expect("viewport copy should be encoded in the final serial suffix");
    let scene_submit = render
        .find("self.submit_compiled_scene_frame(")
        .expect("compiled scene should retain one submission boundary");

    assert!(begin < request);
    assert!(request < terminal_prepare);
    assert!(terminal_prepare < scene_submit);
    assert!(!terminal.contains("encode_output_target_writeback("));
    assert!(terminal[prepare..].contains("product-diagnostic-readback"));
    assert!(!render.contains("request_texture_rgba("));
    assert!(submit.contains("submit_graphics_command_buffers_with_frame_diagnostics_and_surface("));
    assert!(submit.contains("product_diagnostic_frame"));
}

#[test]
fn hzb_cpu_diagnostics_request_readback_only_when_explicitly_enabled() {
    let source = compiled_scene_execution_source();
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
        .expect("enabled HZB diagnostics must use the product diagnostic frame");

    assert!(gate < request);
    let skipped = source[request..]
        .find("} else if hzb_readback_requested {")
        .map(|offset| request + offset)
        .expect("explicit HZB sampling must report a product-frame admission drop");
    assert!(request < skipped);
    assert!(source[skipped..].contains("culler.record_skipped_readback();"));
    assert!(source[request..].contains("backend,"));
}

#[test]
fn compiled_hzb_and_realtime_ibl_share_the_scene_diagnostic_ticket() {
    let source = compiled_scene_execution_source();
    let terminal = include_str!("terminal_frame_packet.rs");
    let begin = source
        .find(".begin_product_diagnostic_readback_scope(frame_generation)")
        .expect("compiled diagnostics should open one product frame");
    let ibl = source
        .find("request_product_gpu_timestamp_readback(")
        .expect("realtime IBL timestamps should use the product diagnostic router");
    let hzb = source
        .find("culler.request_frame_readbacks(")
        .expect("HZB diagnostics should use the product diagnostic router");
    let terminal_prepare = source
        .find("prepare_terminal_frame_packet(")
        .expect("compiled diagnostics should reach the terminal packet");
    let prepare = terminal
        .find("scope.prepare(")
        .expect("all product diagnostics should be encoded into the scene tail");

    assert!(begin < ibl);
    assert!(begin < hzb);
    assert!(ibl < terminal_prepare);
    assert!(hzb < terminal_prepare);
    assert!(terminal[prepare..].contains("product-diagnostic-readback"));
}

#[test]
fn runtime_prepare_readbacks_share_the_product_diagnostic_scope_and_scene_ticket() {
    let render = compiled_scene_execution_source();
    let terminal = include_str!("terminal_frame_packet.rs");
    let submit = include_str!("submit_compiled_scene_frame.rs");
    let begin = render
        .find(".begin_product_diagnostic_readback_scope(frame_generation)")
        .expect("readback collectors must reserve the product diagnostic frame before prepare");
    let runtime_prepare = render
        .find("self.execute_runtime_prepare_passes(")
        .expect("runtime prepare must remain explicit");
    let viewport = render
        .find("backend.enqueue_product_diagnostic_texture_rgba8(")
        .expect("viewport capture must retain highest product readback priority");
    let plugins = render
        .find("advanced_plugin_readbacks.register_product_gpu_readbacks(backend)")
        .expect("runtime prepare readbacks must use the product diagnostic router");
    let hzb = render
        .find("culler.request_frame_readbacks(")
        .expect("HZB diagnostics must share the product diagnostic frame");
    let ibl = render
        .find("request_product_gpu_timestamp_readback(")
        .expect("realtime IBL timestamps must share the product diagnostic frame");
    let artifact = render
        .find("prepare_environment_ibl_runtime_cache_writeback(")
        .expect("IBL artifact sections must share the product diagnostic frame");
    let terminal_prepare = render
        .find("prepare_terminal_frame_packet(")
        .expect("compiled diagnostics must reach the terminal packet");
    let prepare = terminal
        .find("scope.prepare(")
        .expect("product diagnostic copies must be prepared in the serial tail");
    let scene_submit = render
        .find("self.submit_compiled_scene_frame(")
        .expect("compiled scene must retain one submission boundary");
    let query_prepare = terminal
        .find(".finish_and_prepare(")
        .expect("typed query resolve must be prepared in the serial tail");

    assert!(begin < runtime_prepare);
    assert!(runtime_prepare < viewport);
    assert!(viewport < plugins);
    assert!(plugins < hzb);
    assert!(hzb < ibl);
    assert!(ibl < artifact);
    assert!(artifact < terminal_prepare);
    assert!(terminal_prepare < scene_submit);
    assert!(!terminal.contains("encode_output_target_writeback("));
    assert!(prepare < query_prepare);
    assert!(render.contains("product_diagnostic_frame_scope.is_some(),"));
    assert!(submit.contains("submit_graphics_command_buffers_with_frame_diagnostics_and_surface("));
    assert!(submit.contains("product_diagnostic_query_frame"));
    assert!(!render.contains("register_gpu_readbacks(&mut self.readback_queue)"));
}

#[test]
fn empty_taa_reactive_mask_stream_binds_the_shared_black_texture() {
    let source = include_str!("bind_taa_reactive_mask_graph_resource.rs");

    assert!(source.contains("PostProcessGraphResourceNames::TAA_REACTIVE_MASK"));
    assert!(source.contains("taa_reactive_mask_stream().is_empty()"));
    assert!(source.contains("post_process.black_texture_view()"));
    assert!(source.contains("import_borrowed_texture_view"));
}

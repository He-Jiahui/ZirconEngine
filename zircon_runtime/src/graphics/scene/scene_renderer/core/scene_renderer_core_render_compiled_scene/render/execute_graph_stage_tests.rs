use crate::core::framework::render::{
    PostProcessEffectKind, PostProcessPassGraph, PostProcessPassNode, RenderBudgetKey,
    RenderGraphPassProfileMetrics, RenderParticleGpuReadbackOutputs, RenderPluginRendererOutputs,
};
use crate::graphics::pipeline::RenderPassStage;
use crate::graphics::scene::scene_renderer::graph_execution::{
    RenderGraphComputeWorkloadDispatchContext, RenderGraphExecutionRecord,
    RenderGraphExecutionResources,
};
use crate::graphics::scene::scene_renderer::mesh::mesh_pass::MeshDrawReplayStats;
use crate::render_graph::QueueLane;
use zr_rhi_wgpu::{WgpuBufferUploadBatch, WgpuTextureUploadBatch};

use super::{
    RecordedGraphPass, RenderGraphStageExecution, render_profile_metrics_from_mesh_replay_stats,
};

#[test]
fn workload_audit_uses_the_compiled_volumetric_history_descriptor() {
    let source = include_str!("execute_graph_stage.rs");

    assert!(source.contains("history_epilogue_plan()"));
    assert!(source.contains(".volumetric_scattering()"));
    assert!(!source.contains("owned_texture_desc("));
}

#[test]
fn mesh_replay_counter_delta_maps_to_pass_profile_metrics() {
    let before = MeshDrawReplayStats {
        draw_call_count: 3,
        state_change_count: 5,
        bind_skip_count: 2,
        ..MeshDrawReplayStats::default()
    };
    let after = MeshDrawReplayStats {
        draw_call_count: 7,
        state_change_count: 11,
        bind_skip_count: 4,
        ..MeshDrawReplayStats::default()
    };

    assert_eq!(
        render_profile_metrics_from_mesh_replay_stats(Some(before), Some(after)),
        RenderGraphPassProfileMetrics::new(4, 0, 6)
    );
    assert_eq!(
        render_profile_metrics_from_mesh_replay_stats(Some(after), Some(before)),
        RenderGraphPassProfileMetrics::default(),
        "replay counter resets must not underflow the per-pass profile"
    );
    assert_eq!(
        render_profile_metrics_from_mesh_replay_stats(None, Some(after)),
        RenderGraphPassProfileMetrics::default()
    );
}

#[test]
fn stage_execution_records_post_process_graph_through_record_owner() {
    let graph = PostProcessPassGraph::from_ordered_nodes(
        vec![PostProcessPassNode::new(
            "output-transfer",
            PostProcessEffectKind::OutputTransfer,
        )],
        Vec::new(),
        Some("output-transfer".to_string()),
    );
    let mut resources = RenderGraphExecutionResources::new();
    let mut record = RenderGraphExecutionRecord::default();
    let mut plugin_outputs = RenderPluginRendererOutputs::default();
    let mut execution = RenderGraphStageExecution::new(
        &mut resources,
        &mut record,
        &mut plugin_outputs,
        None,
        None,
    );

    execution.record_post_process_graph(&graph);

    assert_eq!(record.post_process_graph(), Some(&graph));
    assert_eq!(
        record.executed_post_process_nodes(),
        &["output-transfer".to_string()]
    );
    assert!(record.executed_passes().is_empty());
}

#[test]
fn stage_execution_commits_pass_owned_results_in_topology_order() {
    let resources = RenderGraphExecutionResources::new();
    let mut record = RenderGraphExecutionRecord::default();
    let mut plugin_outputs = RenderPluginRendererOutputs::default();
    let mut execution =
        RenderGraphStageExecution::new(&resources, &mut record, &mut plugin_outputs, None, None);

    for (pass_name, alive_count) in [("first", 3), ("second", 7)] {
        execution.commit_recorded_pass(
            RecordedGraphPass {
                stage: RenderPassStage::Opaque3d,
                pass_name: pass_name.to_string(),
                executor_id: "mesh.opaque".to_string(),
                queue: QueueLane::Graphics,
                declared_queue: QueueLane::Graphics,
                dependencies: Vec::new(),
                resources: Vec::new(),
                debug_marker: format!("zircon::render_graph::{pass_name}"),
                budget_key: RenderBudgetKey::BasePass,
                cpu_elapsed_micros: 1,
                render_metrics: RenderGraphPassProfileMetrics::default(),
                mesh_replay_stats: MeshDrawReplayStats::default(),
                compute_workload: None,
                dispatch_context: RenderGraphComputeWorkloadDispatchContext::new([1, 1], [1, 1], 0),
                compute_dispatches: Vec::new(),
                motion_vector_camera_status: Default::default(),
                hzb_occlusion_cull_report: None,
                light_grid_report: None,
                taa_reactive_mask_encoding: (0, 0),
                taa_resolve_bind_group_create_count: 0,
                buffer_uploads: WgpuBufferUploadBatch::new(),
                texture_uploads: WgpuTextureUploadBatch::new(),
                screen_space_ui_upload_commits: Vec::new(),
                hzb_occlusion_params_commits: Vec::new(),
                output_target_writeback_report: None,
                plugin_outputs: RenderPluginRendererOutputs {
                    particles: RenderParticleGpuReadbackOutputs {
                        alive_count,
                        ..RenderParticleGpuReadbackOutputs::default()
                    },
                    ..RenderPluginRendererOutputs::default()
                },
                history_writes: Default::default(),
            },
            None,
        );
    }

    assert_eq!(
        record.executed_passes(),
        &["first".to_string(), "second".to_string()]
    );
    assert_eq!(plugin_outputs.particles.alive_count, 7);
}

#[test]
fn graph_pass_uploads_are_pass_local_and_merged_in_recorded_order_without_a_mutex() {
    let stage_source = include_str!("execute_graph_stage.rs");
    let gpu_context_source =
        include_str!("../../../graph_execution/render_pass_execution_context/gpu.rs");

    assert!(gpu_context_source.contains("buffer_uploads: WgpuBufferUploadBatch"));
    assert!(gpu_context_source.contains("take_buffer_uploads"));
    assert!(stage_source.contains("buffer_uploads: WgpuBufferUploadBatch"));
    assert!(stage_source.contains("self.buffer_uploads.append(&mut recorded.buffer_uploads)"));
    assert!(stage_source.contains(".append(&mut recorded.hzb_occlusion_params_commits)"));
    assert!(stage_source.contains("take_hzb_occlusion_params_commits"));
    assert!(!stage_source.contains("Mutex<WgpuBufferUploadBatch>"));
    assert!(!stage_source.contains("Mutex<Vec<HzbOcclusionParamsCommit>>"));
    assert!(stage_source.contains("for recorded_bucket in recorded_buckets"));
    assert!(stage_source.contains("for recorded in recorded_passes"));
}

#[test]
fn graph_stage_execution_receives_one_named_frame_services_argument() {
    let source = include_str!("execute_graph_stage.rs");
    let (_, stage_execution) = source
        .split_once("pub(in crate::graphics::scene::scene_renderer::core::scene_renderer_core_render_compiled_scene) fn execute_graph_stage")
        .expect("stage execution function should remain present");
    let (signature, _) = stage_execution
        .split_once("-> Result<(), GraphicsError>")
        .expect("stage execution function should retain its fallible result");

    assert!(signature.contains("services: RenderGraphPassFrameServices"));
    assert!(!signature.contains("screen_space_ui_renderer: Option"));
    assert!(!signature.contains("post_process_stack: Option"));
}

#[test]
fn graph_stage_execution_forwards_packet_access_identities_to_executors() {
    let source = include_str!("execute_graph_stage.rs");

    assert!(source.contains("pipeline.execution_access_ids_for_pass"));
    assert!(source.contains("access_ids: &'a [RenderGraphResourceAccessId]"));
    assert!(source.contains(".with_compiled_access_ids(pass.id, access_ids)"));
    assert!(source.contains(".with_compute_binding_access_packet(pipeline.graph().compute_binding_access_packet(pass.id))"));
    assert!(source.contains(".with_compute_dispatch_access_packet("));
    assert!(source.contains("pipeline.graph().compute_dispatch_access_packet(pass.id)"));
}

#[test]
fn graph_stage_execution_consumes_packet_batches_before_stage_routing() {
    let source = include_str!("execute_graph_stage.rs");

    assert!(source.contains("for batch in pipeline.execution_batches_for_stage(stage)"));
    assert!(source.contains("pipeline.execution_passes_for_batch(batch)"));
    assert!(source.contains(".filter(|execution_pass| execution_pass.stage == stage)"));
    assert!(!source.contains("for execution_pass in pipeline.execution_passes_for_stage(stage)"));
}

#[test]
fn graph_stage_execution_uses_packet_cursor_for_global_order_and_completion() {
    let source = include_str!("execute_graph_stage.rs");

    assert!(source.contains("execution_cursor: Option<RenderGraphExecutionCursor>"));
    assert!(source.contains("pipeline.begin_execution()"));
    assert!(source.contains(".admit_execution_pass(cursor, graph_pass_index)"));
    assert!(source.contains(".finish_execution(cursor)"));
}

#[test]
fn deferred_mesh_pipeline_context_is_required_independently_from_streamer() {
    let (_, source) = include_str!("execute_graph_stage.rs")
        .rsplit_once("fn execute_graph_pass")
        .expect("graph-stage source should contain the pass assembly function");
    let scene_passes = include_str!("../scene_passes/render_scene_passes.rs");

    assert!(source.contains(
        "if let (Some(mesh_pipelines), Some(mesh_draw_lists)) = (mesh_pipelines, mesh_draw_lists)"
    ));
    assert!(!source.contains(
        "if let (Some(mesh_pipelines), Some(streamer), Some(mesh_draw_lists)) =\n        (mesh_pipelines, streamer, mesh_draw_lists)"
    ));
    assert!(scene_passes.contains("&self.deferred,\n                &mut self.mesh_pipelines,"));
    assert!(scene_passes.contains("RenderPassStage::Deferred,\n                Some(streamer),"));
    assert!(scene_passes.contains("RenderPassStage::Lighting,\n                None,"));
    assert!(scene_passes.contains("mesh_pipelines: &mut MeshPipelineCache,"));
    assert!(!scene_passes.contains("mesh_pipelines: Option<&mut MeshPipelineCache>,"));
    assert!(scene_passes.contains("RenderGraphPassFrameServices {"));
    assert!(scene_passes.contains("ibl_bake_pipeline_cache: Some(ibl_bake_pipeline_cache),"));
}

#[test]
fn render_perf_parallel_recording_is_wired_to_product_stage_and_single_submit_owner() {
    let stage_source = include_str!("execute_graph_stage.rs");
    let render_source = include_str!("render.rs");
    let submit_source = include_str!("submit_compiled_scene_frame.rs");

    assert!(stage_source.contains("ParallelEncoderSet::partition_filtered"));
    assert!(stage_source.contains("record_parallel_with_outputs"));
    assert!(stage_source.contains("registry.supports_parallel_recording(executor_id)"));
    assert!(stage_source.contains("|| mesh_draw_lists.is_some()"));
    let prepared_scope_clone = concat!("prepared.gpu_timestamp_scope", ".clone()");
    let task_scope_clone = concat!("gpu_timestamp_scope", ".clone(),");
    assert!(stage_source.contains(prepared_scope_clone));
    assert!(stage_source.contains(task_scope_clone));
    let timestamp_serialization_guard = concat!("!gpu_", "timestamps_enabled");
    assert!(
        !stage_source.contains(timestamp_serialization_guard),
        "timestamp scopes must not select a different graph-recording policy"
    );
    assert!(stage_source.contains("record_parallel_recording_eligibility"));
    assert!(stage_source.contains("record_parallel_recording_execution"));
    assert!(stage_source.contains("command_encoders.flush_serial_prefix()"));
    assert!(stage_source.contains("parallel bucket references unprepared graph pass index"));
    assert!(!stage_source.contains(".expect("));
    assert!(render_source.contains("command_buffers: command_encoders.finish()"));
    assert!(submit_source.contains("queue.submit(command_buffers)"));
    assert_eq!(submit_source.matches("queue.submit(").count(), 1);
}

#[test]
fn compiled_graph_passes_expose_upload_recording_without_native_queue_authority() {
    let services = include_str!("render_graph_pass_frame_services.rs");
    let stages = include_str!("execute_compiled_scene_graph_stages.rs");
    let stage = include_str!("execute_graph_stage.rs");
    let gpu_context = include_str!("../../../graph_execution/render_pass_execution_context/gpu.rs");
    let gpu_product = gpu_context
        .split_once("#[cfg(test)]")
        .map(|(product, _)| product)
        .expect("GPU pass context should retain a test-constructor boundary");
    let ui_record = include_str!("../../../ui/render/record.rs");
    let particle_context =
        include_str!("../../../graph_execution/render_pass_execution_context/gpu/particle.rs");
    let frame = include_str!("render.rs");

    assert!(!services.contains("wgpu::Queue"));
    assert!(!stages.contains("wgpu::Queue"));
    assert!(!stage.contains("queue: &wgpu::Queue"));
    assert!(!gpu_product.contains("wgpu::Queue"));
    assert!(!ui_record.contains("wgpu::Queue"));
    assert!(!particle_context.contains("wgpu::Queue"));
    assert!(particle_context.contains("RenderPassBufferUploadRecorder"));

    let upload = frame
        .find(".enqueue_copy_resource_upload_batch(")
        .expect("compiled frame resource upload packet");
    let scene_submit = frame
        .find("submit_compiled_scene_frame(")
        .expect("compiled scene submission");
    assert!(upload < scene_submit);
}

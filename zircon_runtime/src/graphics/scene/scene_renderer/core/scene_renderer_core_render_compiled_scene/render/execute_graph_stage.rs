use std::time::{Duration, Instant};

use crate::core::TaskPool;
use crate::core::framework::render::{
    MotionVectorCameraStatus, PostProcessGraphResourceNames, PostProcessPassGraph, RenderBudgetKey,
    RenderGraphPassProfileMetrics, RenderPluginRendererOutputs,
};
use crate::graphics::backend::{GpuPassTimer, GpuPassTimestampScope, GpuPipelineStatisticsTimer};
use crate::graphics::debug_markers::{
    insert_marker, marker_for_render_graph_pass, marker_for_render_pass_stage,
};
use crate::graphics::pipeline::RenderPassStage;
use crate::graphics::pipeline::{CompiledRenderPipeline, CompiledRenderPipelinePassStage};
use crate::graphics::scene::resources::ResourceStreamer;
use crate::graphics::scene::scene_renderer::cluster_dimensions_for_size;
use crate::graphics::scene::scene_renderer::deferred::DeferredSceneResources;
use crate::graphics::scene::scene_renderer::environment::IblBakeWgpuPipelineCache;
use crate::graphics::scene::scene_renderer::environment::ibl_bake_graph_plan::{
    IBL_BAKE_IRRADIANCE_CUBE_EXECUTOR_ID, IBL_BAKE_IRRADIANCE_SH9_EXECUTOR_ID,
    IBL_BAKE_PMREM_EXECUTOR_ID,
};
use crate::graphics::scene::scene_renderer::graph_execution::parallel_encoder_set::ParallelEncoderSet;
use crate::graphics::scene::scene_renderer::graph_execution::{
    FrameCommandEncoderSet, RenderGraphComputeDispatchRecord,
    RenderGraphComputeWorkloadDispatchContext, RenderGraphExecutionRecord,
    RenderGraphExecutionResources, RenderGraphLightGridReport, RenderPassExecutionContext,
    RenderPassExecutorId, RenderPassExecutorRegistry, RenderPassGpuExecutionContext,
    RenderPassMeshCommandLists, RenderPassPostProcessStackContext,
};
use crate::graphics::scene::scene_renderer::hzb::HzbOcclusionCuller;
use crate::graphics::scene::scene_renderer::mesh::MeshPipelineCache;
use crate::graphics::scene::scene_renderer::mesh::mesh_pass::{
    MeshDrawReplayStats, MeshDrawReplayStatsAccumulator,
};
use crate::graphics::scene::scene_renderer::overlay::{
    PreparedOverlayBuffers, ViewportOverlayRenderer,
};
use crate::graphics::scene::scene_renderer::particle::ParticleRenderer;
use crate::graphics::scene::scene_renderer::post_process::execute_post_process_pass_graph;
use crate::graphics::scene::scene_renderer::shadow::atlas::ShadowAtlasResources;
use crate::graphics::scene::scene_renderer::shadow::{ShadowFramePlan, ShadowMapRenderer};
use crate::graphics::scene::scene_renderer::sprite::SpriteRenderer;
use crate::graphics::scene::scene_renderer::ui::ScreenSpaceUiRenderer;
use crate::graphics::types::{GraphicsError, ViewportRenderFrame};
use crate::graphics::visibility::{HzbBuilder, HzbOcclusionCullReport};
use crate::render_graph::{
    CompiledRenderPass, QueueLane, RenderGraphComputeWorkload, RenderGraphPassResourceAccess,
    RenderPassId,
};

use super::super::super::scene_renderer_core::merge_plugin_renderer_outputs;

pub(in crate::graphics::scene::scene_renderer::core::scene_renderer_core_render_compiled_scene)
struct RenderGraphStageExecution
<'a> {
    pub(in crate::graphics::scene::scene_renderer::core::scene_renderer_core_render_compiled_scene) resources:
        &'a RenderGraphExecutionResources,
    pub(in crate::graphics::scene::scene_renderer::core::scene_renderer_core_render_compiled_scene) record:
        &'a mut RenderGraphExecutionRecord,
    pub(in crate::graphics::scene::scene_renderer::core::scene_renderer_core_render_compiled_scene) plugin_outputs:
        &'a mut RenderPluginRendererOutputs,
    gpu_pass_timer: Option<&'a mut GpuPassTimer>,
    gpu_pipeline_statistics_timer: Option<&'a mut GpuPipelineStatisticsTimer>,
}

struct RecordedGraphPass {
    stage: RenderPassStage,
    pass_name: String,
    executor_id: String,
    queue: QueueLane,
    declared_queue: QueueLane,
    dependencies: Vec<RenderPassId>,
    resources: Vec<RenderGraphPassResourceAccess>,
    debug_marker: String,
    budget_key: RenderBudgetKey,
    cpu_elapsed_micros: u64,
    render_metrics: RenderGraphPassProfileMetrics,
    mesh_replay_stats: MeshDrawReplayStats,
    compute_workload: Option<RenderGraphComputeWorkload>,
    dispatch_context: RenderGraphComputeWorkloadDispatchContext,
    compute_dispatches: Vec<RenderGraphComputeDispatchRecord>,
    motion_vector_camera_status: MotionVectorCameraStatus,
    hzb_occlusion_cull_report: Option<HzbOcclusionCullReport>,
    light_grid_report: Option<RenderGraphLightGridReport>,
    taa_reactive_mask_encoding: (usize, u64),
    taa_resolve_bind_group_create_count: usize,
    plugin_outputs: RenderPluginRendererOutputs,
}

struct PreparedStagePass<'a> {
    graph_pass_index: usize,
    stage_entry: &'a CompiledRenderPipelinePassStage,
    pass: &'a CompiledRenderPass,
    gpu_timestamp_scope: Option<GpuPassTimestampScope>,
}

impl<'a> RenderGraphStageExecution<'a> {
    pub(in crate::graphics::scene::scene_renderer::core::scene_renderer_core_render_compiled_scene) fn new(
        resources: &'a RenderGraphExecutionResources,
        record: &'a mut RenderGraphExecutionRecord,
        plugin_outputs: &'a mut RenderPluginRendererOutputs,
        gpu_pass_timer: Option<&'a mut GpuPassTimer>,
        gpu_pipeline_statistics_timer: Option<&'a mut GpuPipelineStatisticsTimer>,
    ) -> Self {
        Self {
            resources,
            record,
            plugin_outputs,
            gpu_pass_timer,
            gpu_pipeline_statistics_timer,
        }
    }

    pub(in crate::graphics::scene::scene_renderer::core::scene_renderer_core_render_compiled_scene) fn record_post_process_graph(
        &mut self,
        graph: &PostProcessPassGraph,
    ) {
        let graph = graph.clone();
        execute_post_process_pass_graph(&graph, &*self.resources, &mut *self.record);
        self.record.set_post_process_graph(graph);
    }

    fn merge_pass_plugin_outputs(&mut self, outputs: RenderPluginRendererOutputs) {
        merge_plugin_renderer_outputs(&mut *self.plugin_outputs, outputs);
    }

    fn commit_recorded_pass(
        &mut self,
        recorded: RecordedGraphPass,
        replay_stats: Option<&MeshDrawReplayStatsAccumulator>,
    ) {
        if let Some(replay_stats) = replay_stats {
            replay_stats.record(recorded.mesh_replay_stats);
        }
        self.merge_pass_plugin_outputs(recorded.plugin_outputs);
        self.record
            .push_pass_profile_with_budget_key_and_compute_dispatches(
                recorded.pass_name.clone(),
                recorded.executor_id.clone(),
                recorded.budget_key,
                recorded.cpu_elapsed_micros,
                recorded.render_metrics,
                &recorded.compute_dispatches,
            );
        self.record.audit_compute_workload(
            &recorded.pass_name,
            &recorded.executor_id,
            recorded.compute_workload.as_ref(),
            recorded.dispatch_context,
            &recorded.compute_dispatches,
        );
        if recorded.motion_vector_camera_status != MotionVectorCameraStatus::NotRequested {
            self.record
                .set_motion_vector_camera_status(recorded.motion_vector_camera_status);
        }
        if let Some(report) = recorded.hzb_occlusion_cull_report {
            self.record.set_hzb_occlusion_cull_report(report);
        }
        if let Some(report) = recorded.light_grid_report {
            self.record.set_light_grid_report(report);
        }
        self.record.add_taa_reactive_mask_encoding(
            recorded.taa_reactive_mask_encoding.0,
            recorded.taa_reactive_mask_encoding.1,
        );
        self.record
            .add_taa_resolve_bind_group_create_count(recorded.taa_resolve_bind_group_create_count);
        self.record
            .push_executed_pass_with_stage_declared_queue_dependencies_resources_and_debug_marker(
                Some(recorded.stage),
                recorded.pass_name,
                recorded.executor_id,
                recorded.queue,
                recorded.declared_queue,
                recorded.dependencies,
                recorded.resources,
                Some(recorded.debug_marker),
            );
        for dispatch in recorded.compute_dispatches {
            self.record.push_compute_dispatch(dispatch);
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub(in crate::graphics::scene::scene_renderer::core::scene_renderer_core_render_compiled_scene) fn execute_graph_stage(
    pipeline: &CompiledRenderPipeline,
    registry: &RenderPassExecutorRegistry,
    stage: RenderPassStage,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    command_encoders: &mut FrameCommandEncoderSet,
    frame: &ViewportRenderFrame,
    scene_bind_group_layout: &wgpu::BindGroupLayout,
    target_format: wgpu::TextureFormat,
    depth_format: wgpu::TextureFormat,
    scene_bind_group: &wgpu::BindGroup,
    mut screen_space_ui_renderer: Option<&mut ScreenSpaceUiRenderer>,
    post_process_stack: Option<RenderPassPostProcessStackContext<'_>>,
    mut overlay_renderer: Option<&mut ViewportOverlayRenderer>,
    prepared_overlays: Option<&PreparedOverlayBuffers>,
    deferred: Option<&DeferredSceneResources>,
    particle_renderer: Option<&ParticleRenderer>,
    sprite_renderer: Option<&SpriteRenderer>,
    streamer: Option<&ResourceStreamer>,
    mut mesh_pipelines: Option<&mut MeshPipelineCache>,
    mut ibl_bake_pipeline_cache: Option<&mut IblBakeWgpuPipelineCache>,
    mesh_draw_lists: Option<RenderPassMeshCommandLists<'_>>,
    hzb_occlusion_culler: Option<&HzbOcclusionCuller>,
    shadow_map_renderer: Option<&ShadowMapRenderer>,
    shadow_atlas_resources: Option<&ShadowAtlasResources>,
    shadow_frame_plan: Option<&ShadowFramePlan>,
    parallel_recording: Option<(&TaskPool, usize)>,
    execution: &mut RenderGraphStageExecution<'_>,
) -> Result<(), GraphicsError> {
    crate::profile_dynamic_scope!("runtime", "render_graph.stage", format!("{stage:?}"));
    let mut prepared_passes = Vec::new();
    for stage_entry in pipeline
        .pass_stages
        .iter()
        .filter(|entry| entry.stage == stage)
    {
        let Some((graph_pass_index, pass)) = pipeline.graph().indexed_pass(stage_entry.pass_id)
        else {
            return Err(GraphicsError::Asset(format!(
                "compiled render pipeline `{}` records stage `{:?}` for missing pass identity {:?} (`{}`)",
                pipeline.name, stage_entry.stage, stage_entry.pass_id, stage_entry.pass_name
            )));
        };
        if pass.name != stage_entry.pass_name {
            return Err(GraphicsError::Asset(format!(
                "compiled render pipeline `{}` maps pass identity {:?} to `{}` but stage metadata names `{}`",
                pipeline.name, stage_entry.pass_id, pass.name, stage_entry.pass_name
            )));
        }
        if pass.culled {
            continue;
        }
        let gpu_timestamp_scope = execution
            .gpu_pass_timer
            .as_deref_mut()
            .and_then(|timer| timer.reserve_pass(&pass.name));
        prepared_passes.push(PreparedStagePass {
            graph_pass_index,
            stage_entry,
            pass,
            gpu_timestamp_scope,
        });
    }

    let ibl_bake_pass_present = prepared_passes.iter().any(|prepared| {
        matches!(
            prepared.pass.executor_id.as_deref(),
            Some(
                IBL_BAKE_PMREM_EXECUTOR_ID
                    | IBL_BAKE_IRRADIANCE_SH9_EXECUTOR_ID
                    | IBL_BAKE_IRRADIANCE_CUBE_EXECUTOR_ID
            )
        )
    });
    let gpu_pipeline_statistics_enabled = execution.gpu_pipeline_statistics_timer.is_some();
    let mutable_recording_owner_present = screen_space_ui_renderer.is_some()
        || overlay_renderer.is_some()
        || mesh_pipelines.is_some()
        || mesh_draw_lists.is_some()
        || ibl_bake_pass_present;
    let all_executors_parallel_safe = prepared_passes.iter().all(|prepared| {
        prepared
            .pass
            .executor_id
            .as_deref()
            .is_some_and(|executor_id| registry.supports_parallel_recording(executor_id))
    });
    let execution_resources = execution.resources;
    if let Some((task_pool, min_passes_per_bucket)) = parallel_recording {
        if !gpu_pipeline_statistics_enabled
            && !mutable_recording_owner_present
            && all_executors_parallel_safe
        {
            let parallel_prepared_passes = prepared_passes
                .iter()
                .map(|prepared| {
                    (
                        prepared.graph_pass_index,
                        prepared.stage_entry,
                        prepared.pass,
                        prepared.gpu_timestamp_scope.clone(),
                    )
                })
                .collect::<Vec<_>>();
            let mut prepared_index_by_graph_pass = vec![None; pipeline.graph().passes().len()];
            for (prepared_index, (graph_pass_index, _, _, _)) in
                parallel_prepared_passes.iter().enumerate()
            {
                prepared_index_by_graph_pass[*graph_pass_index] = Some(prepared_index);
            }
            let parallel_encoders = ParallelEncoderSet::partition_filtered(
                pipeline.graph(),
                min_passes_per_bucket,
                |pass_index, _| prepared_index_by_graph_pass[pass_index].is_some(),
            );
            if parallel_encoders.should_record_parallel(true, task_pool) {
                let eligible_bucket_count = parallel_encoders.buckets().len();
                execution
                    .record
                    .record_parallel_recording_eligibility(eligible_bucket_count);
                command_encoders.flush_serial_prefix();
                let recorded_buckets = parallel_encoders.record_parallel_with_outputs(
                    device,
                    task_pool,
                    |bucket, encoder| {
                        let mut recorded = Vec::with_capacity(bucket.pass_count());
                        for pass_index in bucket.pass_indices() {
                            let prepared_index = prepared_index_by_graph_pass[*pass_index].expect(
                                "parallel encoder bucket must reference a prepared stage pass",
                            );
                            let (_, stage_entry, pass, gpu_timestamp_scope) =
                                &parallel_prepared_passes[prepared_index];
                            recorded.push(execute_graph_pass(
                                pipeline,
                                registry,
                                stage_entry,
                                pass,
                                device,
                                queue,
                                encoder,
                                frame,
                                scene_bind_group_layout,
                                target_format,
                                depth_format,
                                scene_bind_group,
                                None,
                                post_process_stack,
                                None,
                                prepared_overlays,
                                deferred,
                                particle_renderer,
                                sprite_renderer,
                                streamer,
                                None,
                                None,
                                None,
                                hzb_occlusion_culler,
                                shadow_map_renderer,
                                shadow_atlas_resources,
                                shadow_frame_plan,
                                execution_resources,
                                None,
                                gpu_timestamp_scope.clone(),
                            )?);
                        }
                        Ok::<_, GraphicsError>(recorded)
                    },
                )?;
                let executed_bucket_count = recorded_buckets.len();
                for recorded_bucket in recorded_buckets {
                    let (command_buffer, recorded_passes) = recorded_bucket.into_parts();
                    command_encoders.append_parallel_buffers([command_buffer]);
                    for recorded in recorded_passes {
                        execution.commit_recorded_pass(
                            recorded,
                            mesh_draw_lists.map(|lists| lists.replay_stats),
                        );
                    }
                }
                execution
                    .record
                    .record_parallel_recording_execution(executed_bucket_count);
                return Ok(());
            }
        }
    }

    let encoder = command_encoders.serial_encoder(device);
    for prepared in prepared_passes {
        let recorded = execute_graph_pass(
            pipeline,
            registry,
            prepared.stage_entry,
            prepared.pass,
            device,
            queue,
            encoder,
            frame,
            scene_bind_group_layout,
            target_format,
            depth_format,
            scene_bind_group,
            screen_space_ui_renderer.as_deref_mut(),
            post_process_stack,
            overlay_renderer.as_deref_mut(),
            prepared_overlays,
            deferred,
            particle_renderer,
            sprite_renderer,
            streamer,
            ibl_bake_pipeline_cache.as_deref_mut(),
            mesh_pipelines.as_deref_mut(),
            mesh_draw_lists,
            hzb_occlusion_culler,
            shadow_map_renderer,
            shadow_atlas_resources,
            shadow_frame_plan,
            execution_resources,
            execution.gpu_pipeline_statistics_timer.as_deref_mut(),
            prepared.gpu_timestamp_scope,
        )?;
        execution.commit_recorded_pass(recorded, mesh_draw_lists.map(|lists| lists.replay_stats));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::core::framework::render::{
        PostProcessEffectKind, PostProcessPassGraph, PostProcessPassNode, RenderBudgetKey,
        RenderGraphPassProfileMetrics, RenderParticleGpuReadbackOutputs,
        RenderPluginRendererOutputs,
    };
    use crate::graphics::pipeline::RenderPassStage;
    use crate::graphics::scene::scene_renderer::graph_execution::{
        RenderGraphComputeWorkloadDispatchContext, RenderGraphExecutionRecord,
        RenderGraphExecutionResources,
    };
    use crate::graphics::scene::scene_renderer::mesh::mesh_pass::MeshDrawReplayStats;
    use crate::render_graph::QueueLane;

    use super::{
        RecordedGraphPass, RenderGraphStageExecution, render_profile_metrics_from_mesh_replay_stats,
    };

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
        let mut execution = RenderGraphStageExecution::new(
            &resources,
            &mut record,
            &mut plugin_outputs,
            None,
            None,
        );

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
                    dispatch_context: RenderGraphComputeWorkloadDispatchContext::new(
                        [1, 1],
                        [1, 1],
                        0,
                    ),
                    compute_dispatches: Vec::new(),
                    motion_vector_camera_status: Default::default(),
                    hzb_occlusion_cull_report: None,
                    light_grid_report: None,
                    taa_reactive_mask_encoding: (0, 0),
                    taa_resolve_bind_group_create_count: 0,
                    plugin_outputs: RenderPluginRendererOutputs {
                        particles: RenderParticleGpuReadbackOutputs {
                            alive_count,
                            ..RenderParticleGpuReadbackOutputs::default()
                        },
                        ..RenderPluginRendererOutputs::default()
                    },
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
        assert!(
            scene_passes.contains("&self.deferred,\n                &mut self.mesh_pipelines,")
        );
        assert!(
            scene_passes.contains("RenderPassStage::Deferred,\n                Some(streamer),")
        );
        assert!(scene_passes.contains("RenderPassStage::Lighting,\n                None,"));
        assert!(scene_passes.contains("mesh_pipelines: &mut MeshPipelineCache,"));
        assert!(!scene_passes.contains("mesh_pipelines: Option<&mut MeshPipelineCache>,"));
        assert!(!scene_passes.contains("IblBakeWgpuPipelineCache"));
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
        assert!(render_source.contains("command_buffers: command_encoders.finish()"));
        assert!(submit_source.contains("queue.submit(command_buffers)"));
        assert_eq!(submit_source.matches("queue.submit(").count(), 1);
    }
}

#[allow(clippy::too_many_arguments)]
fn execute_graph_pass(
    pipeline: &CompiledRenderPipeline,
    registry: &RenderPassExecutorRegistry,
    stage_entry: &CompiledRenderPipelinePassStage,
    pass: &CompiledRenderPass,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    encoder: &mut wgpu::CommandEncoder,
    frame: &ViewportRenderFrame,
    scene_bind_group_layout: &wgpu::BindGroupLayout,
    target_format: wgpu::TextureFormat,
    depth_format: wgpu::TextureFormat,
    scene_bind_group: &wgpu::BindGroup,
    screen_space_ui_renderer: Option<&mut ScreenSpaceUiRenderer>,
    post_process_stack: Option<RenderPassPostProcessStackContext<'_>>,
    overlay_renderer: Option<&mut ViewportOverlayRenderer>,
    prepared_overlays: Option<&PreparedOverlayBuffers>,
    deferred: Option<&DeferredSceneResources>,
    particle_renderer: Option<&ParticleRenderer>,
    sprite_renderer: Option<&SpriteRenderer>,
    streamer: Option<&ResourceStreamer>,
    ibl_bake_pipeline_cache: Option<&mut IblBakeWgpuPipelineCache>,
    mesh_pipelines: Option<&mut MeshPipelineCache>,
    mesh_draw_lists: Option<RenderPassMeshCommandLists<'_>>,
    hzb_occlusion_culler: Option<&HzbOcclusionCuller>,
    shadow_map_renderer: Option<&ShadowMapRenderer>,
    shadow_atlas_resources: Option<&ShadowAtlasResources>,
    shadow_frame_plan: Option<&ShadowFramePlan>,
    resources: &RenderGraphExecutionResources,
    pipeline_statistics_timer: Option<&mut GpuPipelineStatisticsTimer>,
    gpu_timestamp_scope: Option<GpuPassTimestampScope>,
) -> Result<RecordedGraphPass, GraphicsError> {
    if let Some(marker) = marker_for_render_pass_stage(stage_entry.stage) {
        insert_marker(encoder, marker);
    }
    let pass_debug_marker = marker_for_render_graph_pass(&pass.name);
    insert_marker(encoder, &pass_debug_marker);
    crate::profile_dynamic_scope!("runtime", "render_graph.pass", pass.name.clone());
    let executor_id = pass.executor_id.as_ref().ok_or_else(|| {
        GraphicsError::Asset(format!("render pass `{}` has no executor id", pass.name))
    })?;
    let executor_id = RenderPassExecutorId::new(executor_id.clone());
    if let Some(scope) = gpu_timestamp_scope.as_ref() {
        scope.begin(encoder);
    }
    let mut pass_plugin_outputs = RenderPluginRendererOutputs::default();
    let pass_mesh_replay_stats = MeshDrawReplayStatsAccumulator::default();
    let mesh_draw_lists =
        mesh_draw_lists.map(|lists| lists.with_replay_stats(&pass_mesh_replay_stats));
    let mut gpu = RenderPassGpuExecutionContext::new(
        device,
        queue,
        encoder,
        frame,
        scene_bind_group_layout,
        target_format,
        depth_format,
        scene_bind_group,
        resources,
        &mut pass_plugin_outputs,
        screen_space_ui_renderer,
    )
    .with_half_resolution_transparency_depth_sigma(
        pipeline.half_resolution_transparency_depth_sigma(),
    );
    if let Some(pipeline_statistics_timer) = pipeline_statistics_timer {
        gpu = gpu.with_pipeline_statistics_timer(pipeline_statistics_timer);
    }
    gpu.streamer = streamer;
    if let Some(ibl_bake_pipeline_cache) = ibl_bake_pipeline_cache {
        gpu = gpu.with_ibl_bake_pipeline_cache(ibl_bake_pipeline_cache);
    }
    if let Some(shadow_atlas_resources) = shadow_atlas_resources {
        gpu = gpu.with_shadow_atlas_resources(shadow_atlas_resources);
    }
    if let Some(shadow_frame_plan) = shadow_frame_plan {
        gpu = gpu.with_shadow_frame_plan(shadow_frame_plan);
    }
    if let Some(post_process_stack) = post_process_stack {
        gpu = gpu.with_post_process_stack_context(post_process_stack);
    }
    if let Some(overlay_renderer) = overlay_renderer {
        gpu = if let Some(prepared_overlays) = prepared_overlays {
            gpu.with_overlay_renderer(overlay_renderer, prepared_overlays)
        } else {
            gpu.with_preview_sky_renderer(overlay_renderer)
        };
    }
    if let Some(shadow_map_renderer) = shadow_map_renderer {
        gpu = if let Some(mesh_draw_lists) = mesh_draw_lists {
            gpu.with_shadow_map_renderer(shadow_map_renderer, mesh_draw_lists)
        } else {
            gpu.with_shadow_receiver(shadow_map_renderer)
        };
    }
    if let (Some(sprite_renderer), Some(streamer)) = (sprite_renderer, streamer) {
        gpu = gpu.with_sprite_renderer(sprite_renderer, streamer);
    }
    if let (Some(deferred), Some(mesh_draw_lists)) = (deferred, mesh_draw_lists) {
        gpu = if let Some(streamer) = streamer {
            gpu.with_deferred_renderer(deferred, streamer, mesh_draw_lists)
        } else {
            gpu.with_deferred_lighting_renderer(deferred, mesh_draw_lists)
        };
    }
    if let Some(particle_renderer) = particle_renderer {
        gpu = gpu.with_particle_renderer(particle_renderer);
    }
    if let (Some(mesh_pipelines), Some(mesh_draw_lists)) = (mesh_pipelines, mesh_draw_lists) {
        gpu = gpu.with_mesh_renderer(mesh_pipelines, mesh_draw_lists);
    }
    if let Some(hzb_occlusion_culler) = hzb_occlusion_culler {
        gpu = gpu.with_hzb_occlusion_culler(hzb_occlusion_culler);
    }
    let mut context =
        RenderPassExecutionContext::with_declared_graph_metadata_dependencies_and_resources(
            pass.name.clone(),
            executor_id.clone(),
            pass.queue,
            pass.declared_queue,
            pass.flags,
            pass.dependencies.clone(),
            pass.resources.clone(),
        )
        .with_resource_resolver(pipeline.graph(), pass.id)
        .with_compute_workload(pass.compute_workload.as_ref())
        .with_compute_pass_metadata(pass.compute_pass_metadata.as_ref())
        .with_resource_streamer(streamer)
        .with_gpu(gpu);

    let profile_started = Instant::now();
    let execute_result = registry.execute(&mut context);
    let cpu_elapsed_micros = duration_to_micros(profile_started.elapsed());
    let mesh_replay_stats = pass_mesh_replay_stats.stats();
    let render_metrics = render_profile_metrics_from_mesh_replay_stats(
        mesh_draw_lists.map(|_| MeshDrawReplayStats::default()),
        mesh_draw_lists.map(|_| mesh_replay_stats),
    );
    let (
        compute_dispatches,
        motion_vector_camera_status,
        hzb_occlusion_cull_report,
        light_grid_report,
        taa_reactive_mask_encoding,
        taa_resolve_bind_group_create_count,
    ) = context
        .gpu_mut()
        .map(|gpu| {
            (
                gpu.take_compute_dispatches(),
                gpu.motion_vector_camera_status(),
                gpu.take_hzb_occlusion_cull_report(),
                gpu.take_light_grid_report(),
                gpu.taa_reactive_mask_encoding(),
                gpu.taa_resolve_bind_group_create_count(),
            )
        })
        .unwrap_or_default();
    drop(context);
    if let Some(scope) = gpu_timestamp_scope.as_ref() {
        scope.end(encoder);
    }
    execute_result.map_err(GraphicsError::Asset)?;
    let cluster_grid_size = cluster_dimensions_for_size(frame.viewport_size);
    let hzb_plan = HzbBuilder::new(frame.extract.view.effective_render_size()).build_plan();
    let hzb_occlusion_indirect_arg_count = mesh_draw_lists
        .map(|lists| lists.occlusion_cull_candidate_arg_count())
        .unwrap_or(0);
    let mut dispatch_context = RenderGraphComputeWorkloadDispatchContext::new(
        [cluster_grid_size.x, cluster_grid_size.y],
        [hzb_plan.hzb_size.x, hzb_plan.hzb_size.y],
        hzb_occlusion_indirect_arg_count,
    );
    if let Some(desc) =
        resources.owned_texture_desc(PostProcessGraphResourceNames::VOLUMETRIC_SCATTERING)
    {
        dispatch_context =
            dispatch_context.with_froxel_grid_size([desc.width, desc.height, desc.depth]);
    }
    if let Some(report) = hzb_occlusion_cull_report {
        dispatch_context =
            dispatch_context.with_indirect_args_dispatch_group_count(report.dispatch_group_count);
    }
    Ok(RecordedGraphPass {
        stage: stage_entry.stage,
        pass_name: pass.name.clone(),
        executor_id: executor_id.as_str().to_string(),
        queue: pass.queue,
        declared_queue: pass.declared_queue,
        dependencies: pass.dependencies.clone(),
        resources: pass.resources.clone(),
        debug_marker: pass_debug_marker,
        budget_key: stage_entry.stage.frame_profile_budget_key(),
        cpu_elapsed_micros,
        render_metrics,
        mesh_replay_stats,
        compute_workload: pass.compute_workload.clone(),
        dispatch_context,
        compute_dispatches,
        motion_vector_camera_status,
        hzb_occlusion_cull_report,
        light_grid_report,
        taa_reactive_mask_encoding,
        taa_resolve_bind_group_create_count,
        plugin_outputs: pass_plugin_outputs,
    })
}

fn duration_to_micros(duration: Duration) -> u64 {
    duration.as_micros().min(u128::from(u64::MAX)) as u64
}

fn render_profile_metrics_from_mesh_replay_stats(
    before: Option<MeshDrawReplayStats>,
    after: Option<MeshDrawReplayStats>,
) -> RenderGraphPassProfileMetrics {
    let Some((before, after)) = before.zip(after) else {
        return RenderGraphPassProfileMetrics::default();
    };
    RenderGraphPassProfileMetrics::new(
        after.draw_call_count.saturating_sub(before.draw_call_count),
        0,
        after
            .state_change_count
            .saturating_sub(before.state_change_count),
    )
}

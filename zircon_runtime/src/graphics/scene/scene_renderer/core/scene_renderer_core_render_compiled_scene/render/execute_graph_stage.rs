use std::time::{Duration, Instant};

use crate::core::framework::render::{
    MotionVectorCameraStatus, PostProcessPassGraph, RenderBudgetKey,
    RenderCameraTargetWritebackReport, RenderGraphPassProfileMetrics,
    RenderPassNativeResourceCreateMetrics, RenderPipelinePhase, RenderPluginRendererOutputs,
};
use crate::graphics::backend::{GpuPassTimer, GpuPassTimestampScope, GpuPipelineStatisticsTimer};
use crate::graphics::debug_markers::{
    insert_marker, marker_for_render_graph_pass, marker_for_render_pass_stage,
};
use crate::graphics::pipeline::RenderPassStage;
use crate::graphics::pipeline::{CompiledRenderPipeline, RenderGraphExecutionCursor};
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
    RenderGraphComputeDispatchRecord, RenderGraphComputeWorkloadDispatchContext,
    RenderGraphExecutionRecord, RenderGraphExecutionResources, RenderGraphLightGridReport,
    RenderPassExecutionContext, RenderPassExecutorId, RenderPassExecutorRegistry,
    RenderPassGpuExecutionContext, RenderPassMeshCommandLists, RenderPassPostProcessStackContext,
};
use crate::graphics::scene::scene_renderer::history::SceneHistoryWriteIntent;
use crate::graphics::scene::scene_renderer::hzb::{HzbOcclusionCuller, HzbOcclusionParamsCommit};
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
use crate::graphics::scene::scene_renderer::ui::{
    ScreenSpaceUiPreparedUpload, ScreenSpaceUiRenderer,
};
use crate::graphics::types::{GraphicsError, ViewportRenderFrame};
use crate::graphics::visibility::{HzbBuilder, HzbOcclusionCullReport};
use crate::render_graph::{
    CompiledRenderPass, QueueLane, RenderGraphComputeWorkload, RenderGraphPassResourceAccess,
    RenderGraphResourceAccessId, RenderPassId,
};
use zr_rhi_wgpu::{WgpuBufferUploadBatch, WgpuTextureUploadBatch};

use super::super::super::scene_renderer_core::merge_plugin_renderer_outputs;
use super::RenderGraphPassFrameServices;

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
    buffer_uploads: WgpuBufferUploadBatch,
    texture_uploads: WgpuTextureUploadBatch,
    screen_space_ui_upload_commits: Vec<ScreenSpaceUiPreparedUpload>,
    hzb_occlusion_params_commits: Vec<HzbOcclusionParamsCommit>,
    output_target_writeback_plan: RenderCameraTargetWritebackReport,
    output_target_writeback_report: Option<RenderCameraTargetWritebackReport>,
    graph_pass_coverage: Option<Vec<u8>>,
    execution_cursor: Option<RenderGraphExecutionCursor>,
    history_writes: SceneHistoryWriteIntent,
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
    native_resource_creates: RenderPassNativeResourceCreateMetrics,
    mesh_replay_stats: MeshDrawReplayStats,
    compute_workload: Option<RenderGraphComputeWorkload>,
    dispatch_context: RenderGraphComputeWorkloadDispatchContext,
    compute_dispatches: Vec<RenderGraphComputeDispatchRecord>,
    motion_vector_camera_status: MotionVectorCameraStatus,
    hzb_occlusion_cull_report: Option<HzbOcclusionCullReport>,
    light_grid_report: Option<RenderGraphLightGridReport>,
    taa_reactive_mask_encoding: (usize, u64),
    taa_resolve_bind_group_create_count: usize,
    buffer_uploads: WgpuBufferUploadBatch,
    texture_uploads: WgpuTextureUploadBatch,
    screen_space_ui_upload_commits: Vec<ScreenSpaceUiPreparedUpload>,
    hzb_occlusion_params_commits: Vec<HzbOcclusionParamsCommit>,
    output_target_writeback_report: Option<RenderCameraTargetWritebackReport>,
    plugin_outputs: RenderPluginRendererOutputs,
    history_writes: SceneHistoryWriteIntent,
}

struct PreparedStagePass<'a> {
    graph_pass_index: usize,
    execution_pass: &'a crate::graphics::pipeline::RenderGraphExecutionPass,
    pass: &'a CompiledRenderPass,
    access_ids: &'a [RenderGraphResourceAccessId],
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
            buffer_uploads: WgpuBufferUploadBatch::new(),
            texture_uploads: WgpuTextureUploadBatch::new(),
            screen_space_ui_upload_commits: Vec::new(),
            hzb_occlusion_params_commits: Vec::new(),
            output_target_writeback_plan: Default::default(),
            output_target_writeback_report: None,
            graph_pass_coverage: None,
            execution_cursor: None,
            history_writes: SceneHistoryWriteIntent::default(),
        }
    }

    pub(in crate::graphics::scene::scene_renderer::core::scene_renderer_core_render_compiled_scene) fn admit_graph_pass(
        &mut self,
        pipeline: &CompiledRenderPipeline,
        graph_pass_index: usize,
        expected_batch_index: usize,
    ) -> Result<(), GraphicsError> {
        let pass = pipeline
            .graph()
            .passes()
            .get(graph_pass_index)
            .ok_or_else(|| {
                GraphicsError::Asset(format!(
                    "compiled render pipeline `{}` execution references missing graph pass index {graph_pass_index}",
                    pipeline.name
                ))
            })?;
        if pass.culled {
            return Err(GraphicsError::Asset(format!(
                "compiled render pipeline `{}` execution attempted to admit culled graph pass `{}` at index {graph_pass_index}",
                pipeline.name, pass.name
            )));
        }
        let Some(batch_index) = pipeline.execution_batch_index_for_pass(graph_pass_index) else {
            return Err(GraphicsError::Asset(format!(
                "compiled render pipeline `{}` execution pass `{}` at index {graph_pass_index} has no live execution batch",
                pipeline.name, pass.name
            )));
        };
        if batch_index != expected_batch_index {
            return Err(GraphicsError::Asset(format!(
                "compiled render pipeline `{}` execution pass `{}` at index {graph_pass_index} belongs to batch {batch_index}, but stage routing supplied batch {expected_batch_index}",
                pipeline.name, pass.name
            )));
        }
        let cursor = self
            .execution_cursor
            .get_or_insert_with(|| pipeline.begin_execution());
        pipeline
            .admit_execution_pass(cursor, graph_pass_index)
            .map_err(GraphicsError::Asset)?;
        let coverage = self
            .graph_pass_coverage
            .get_or_insert_with(|| vec![0; pipeline.graph().passes().len()]);
        if coverage[graph_pass_index] != 0 {
            return Err(GraphicsError::Asset(format!(
                "compiled render pipeline `{}` execution admitted graph pass `{}` at index {graph_pass_index} more than once",
                pipeline.name, pass.name
            )));
        }
        coverage[graph_pass_index] = 1;
        Ok(())
    }

    pub(in crate::graphics::scene::scene_renderer::core::scene_renderer_core_render_compiled_scene) fn validate_graph_execution(
        &self,
        pipeline: &CompiledRenderPipeline,
    ) -> Result<(), GraphicsError> {
        for (graph_pass_index, pass) in pipeline.graph().passes().iter().enumerate() {
            if pass.culled {
                continue;
            }
            let admitted = self
                .graph_pass_coverage
                .as_ref()
                .and_then(|coverage| coverage.get(graph_pass_index))
                .copied()
                .unwrap_or(0);
            if admitted != 1 {
                return Err(GraphicsError::Asset(format!(
                    "compiled render pipeline `{}` did not execute live graph pass `{}` at index {graph_pass_index} exactly once",
                    pipeline.name, pass.name
                )));
            }
        }
        if let Some(cursor) = self.execution_cursor {
            pipeline
                .finish_execution(cursor)
                .map_err(GraphicsError::Asset)?;
        }
        Ok(())
    }

    pub(in crate::graphics::scene::scene_renderer::core::scene_renderer_core_render_compiled_scene) fn with_output_target_writeback_plan(
        mut self,
        plan: RenderCameraTargetWritebackReport,
    ) -> Self {
        self.output_target_writeback_plan = plan;
        self
    }

    pub(in crate::graphics::scene::scene_renderer::core::scene_renderer_core_render_compiled_scene) fn take_buffer_uploads(
        &mut self,
    ) -> WgpuBufferUploadBatch {
        std::mem::take(&mut self.buffer_uploads)
    }

    pub(in crate::graphics::scene::scene_renderer::core::scene_renderer_core_render_compiled_scene) fn take_texture_uploads(
        &mut self,
    ) -> WgpuTextureUploadBatch {
        std::mem::take(&mut self.texture_uploads)
    }

    pub(in crate::graphics::scene::scene_renderer::core::scene_renderer_core_render_compiled_scene) fn append_buffer_uploads(
        &mut self,
        uploads: &mut WgpuBufferUploadBatch,
    ) {
        self.buffer_uploads.append(uploads);
    }

    pub(in crate::graphics::scene::scene_renderer::core::scene_renderer_core_render_compiled_scene) fn take_hzb_occlusion_params_commits(
        &mut self,
    ) -> Vec<HzbOcclusionParamsCommit> {
        std::mem::take(&mut self.hzb_occlusion_params_commits)
    }

    pub(in crate::graphics::scene::scene_renderer::core::scene_renderer_core_render_compiled_scene) fn take_screen_space_ui_upload_commits(
        &mut self,
    ) -> Vec<ScreenSpaceUiPreparedUpload> {
        std::mem::take(&mut self.screen_space_ui_upload_commits)
    }

    pub(in crate::graphics::scene::scene_renderer::core::scene_renderer_core_render_compiled_scene) fn take_output_target_writeback_report(
        &mut self,
    ) -> Option<RenderCameraTargetWritebackReport> {
        self.output_target_writeback_report.take()
    }

    pub(in crate::graphics::scene::scene_renderer::core::scene_renderer_core_render_compiled_scene) const fn history_writes(
        &self,
    ) -> SceneHistoryWriteIntent {
        self.history_writes
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
        mut recorded: RecordedGraphPass,
        replay_stats: Option<&MeshDrawReplayStatsAccumulator>,
    ) {
        self.buffer_uploads.append(&mut recorded.buffer_uploads);
        self.texture_uploads.append(recorded.texture_uploads);
        self.screen_space_ui_upload_commits
            .append(&mut recorded.screen_space_ui_upload_commits);
        self.hzb_occlusion_params_commits
            .append(&mut recorded.hzb_occlusion_params_commits);
        if let Some(report) = recorded.output_target_writeback_report {
            self.output_target_writeback_report = Some(report);
        }
        if let Some(replay_stats) = replay_stats {
            replay_stats.record(recorded.mesh_replay_stats);
        }
        self.merge_pass_plugin_outputs(recorded.plugin_outputs);
        self.history_writes.merge(recorded.history_writes);
        self.record
            .push_pass_profile_with_budget_key_native_resources_and_compute_dispatches(
                recorded.pass_name.clone(),
                recorded.executor_id.clone(),
                recorded.budget_key,
                recorded.cpu_elapsed_micros,
                recorded.render_metrics,
                recorded.native_resource_creates,
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
    services: RenderGraphPassFrameServices<'_>,
    execution: &mut RenderGraphStageExecution<'_>,
) -> Result<(), GraphicsError> {
    let RenderGraphPassFrameServices {
        device,
        command_encoders,
        frame,
        scene_bind_group_layout,
        target_format,
        depth_format,
        scene_bind_group,
        surface_frame,
        mut screen_space_ui_renderer,
        post_process_stack,
        mut overlay_renderer,
        prepared_overlays,
        deferred,
        particle_renderer,
        sprite_renderer,
        streamer,
        mut ibl_bake_pipeline_cache,
        mut mesh_pipelines,
        mesh_draw_lists,
        hzb_occlusion_culler,
        shadow_map_renderer,
        shadow_atlas_resources,
        shadow_frame_plan,
        parallel_recording,
    } = services;
    crate::profile_dynamic_scope!("runtime", "render_graph.stage", format!("{stage:?}"));
    let mut prepared_passes = Vec::new();
    // The immutable packet's batches are the execution authority. Stage is a
    // service-routing filter only; it must not recreate graph ordering or
    // bypass culling/queue boundaries by reading an authored pass list.
    for (batch_index, batch) in pipeline.execution_batches_with_indices_for_stage(stage) {
        for execution_pass in pipeline
            .execution_passes_for_batch(batch)
            .filter(|execution_pass| execution_pass.stage == stage)
        {
            let Some(pass) = pipeline
                .graph()
                .passes()
                .get(execution_pass.graph_pass_index)
            else {
                return Err(GraphicsError::Asset(format!(
                    "compiled render pipeline `{}` execution packet references missing graph pass index {}",
                    pipeline.name, execution_pass.graph_pass_index
                )));
            };
            let Some(access_ids) =
                pipeline.execution_access_ids_for_pass(execution_pass.graph_pass_index)
            else {
                return Err(GraphicsError::Asset(format!(
                    "compiled render pipeline `{}` execution packet references missing access identities for graph pass index {}",
                    pipeline.name, execution_pass.graph_pass_index
                )));
            };
            if access_ids.len() != pass.resources.len() {
                return Err(GraphicsError::Asset(format!(
                    "compiled render pipeline `{}` execution packet access identity count {} differs from graph pass `{}` resource access count {}",
                    pipeline.name,
                    access_ids.len(),
                    pass.name,
                    pass.resources.len()
                )));
            }
            if pass.culled {
                continue;
            }
            execution.admit_graph_pass(pipeline, execution_pass.graph_pass_index, batch_index)?;
            let gpu_timestamp_scope = execution
                .gpu_pass_timer
                .as_deref_mut()
                .and_then(|timer| timer.reserve_pass(&pass.name));
            prepared_passes.push(PreparedStagePass {
                graph_pass_index: execution_pass.graph_pass_index,
                execution_pass,
                pass,
                access_ids,
                gpu_timestamp_scope,
            });
        }
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
        || surface_frame.is_some()
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
    let output_target_writeback_plan = execution.output_target_writeback_plan;
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
                        prepared.execution_pass,
                        prepared.pass,
                        prepared.access_ids,
                        prepared.gpu_timestamp_scope.clone(),
                    )
                })
                .collect::<Vec<_>>();
            let mut prepared_index_by_graph_pass = vec![None; pipeline.graph().passes().len()];
            for (prepared_index, (graph_pass_index, _, _, _, _)) in
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
                            let prepared_index = prepared_index_by_graph_pass
                                .get(*pass_index)
                                .and_then(|prepared_index| *prepared_index)
                                .ok_or_else(|| {
                                    GraphicsError::Asset(format!(
                                        "compiled render pipeline `{}` parallel bucket references unprepared graph pass index {pass_index}",
                                        pipeline.name
                                    ))
                                })?;
                            let (_, execution_pass, pass, access_ids, gpu_timestamp_scope) =
                                &parallel_prepared_passes[prepared_index];
                            recorded.push(execute_graph_pass(
                                pipeline,
                                registry,
                                execution_pass.stage,
                                pass,
                                access_ids,
                                device,
                                encoder,
                                frame,
                                output_target_writeback_plan,
                                scene_bind_group_layout,
                                target_format,
                                depth_format,
                                scene_bind_group,
                                None,
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
            prepared.execution_pass.stage,
            prepared.pass,
            prepared.access_ids,
            device,
            encoder,
            frame,
            output_target_writeback_plan,
            scene_bind_group_layout,
            target_format,
            depth_format,
            scene_bind_group,
            surface_frame,
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
#[path = "execute_graph_stage_tests.rs"]
mod tests;

#[allow(clippy::too_many_arguments)]
fn execute_graph_pass(
    pipeline: &CompiledRenderPipeline,
    registry: &RenderPassExecutorRegistry,
    stage: RenderPassStage,
    pass: &CompiledRenderPass,
    access_ids: &[RenderGraphResourceAccessId],
    device: &wgpu::Device,
    encoder: &mut wgpu::CommandEncoder,
    frame: &ViewportRenderFrame,
    output_target_writeback_plan: RenderCameraTargetWritebackReport,
    scene_bind_group_layout: &wgpu::BindGroupLayout,
    target_format: wgpu::TextureFormat,
    depth_format: wgpu::TextureFormat,
    scene_bind_group: &wgpu::BindGroup,
    surface_frame: Option<(
        &crate::graphics::backend::ViewportSurface,
        &zr_rhi_wgpu::WgpuNativeSurfaceFrameTarget,
    )>,
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
    if let Some(marker) = marker_for_render_pass_stage(stage) {
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
    )
    .with_surface_frame(surface_frame)
    .with_output_target_writeback_plan(output_target_writeback_plan);
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
    let context =
        RenderPassExecutionContext::with_declared_graph_metadata_dependencies_and_resources(
            pass.name.clone(),
            executor_id.clone(),
            pass.queue,
            pass.declared_queue,
            pass.flags,
            pass.dependencies.clone(),
            pass.resources.clone(),
        )
        .with_compiled_access_ids(pass.id, access_ids)
        .map_err(GraphicsError::Asset)?;
    let mut context = context
        .with_resource_resolver(pipeline.graph(), pass.id)
        .with_compute_workload(pass.compute_workload.as_ref())
        .with_compute_pass_metadata(pass.compute_pass_metadata.as_ref())
        .with_compute_binding_access_packet(pipeline.graph().compute_binding_access_packet(pass.id))
        .with_compute_dispatch_access_packet(
            pipeline.graph().compute_dispatch_access_packet(pass.id),
        )
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
        surface_present_error,
        output_target_writeback_error,
        output_target_writeback_report,
        compute_dispatches,
        motion_vector_camera_status,
        hzb_occlusion_cull_report,
        light_grid_report,
        native_resource_creates,
        taa_reactive_mask_encoding,
        taa_resolve_bind_group_create_count,
        buffer_uploads,
        texture_uploads,
        screen_space_ui_upload_commits,
        hzb_occlusion_params_commits,
        history_writes,
    ) = context
        .gpu_mut()
        .map(|gpu| {
            (
                gpu.take_surface_present_error(),
                gpu.take_output_target_writeback_error(),
                gpu.take_output_target_writeback_report(),
                gpu.take_compute_dispatches(),
                gpu.motion_vector_camera_status(),
                gpu.take_hzb_occlusion_cull_report(),
                gpu.take_light_grid_report(),
                gpu.take_native_resource_creates(),
                gpu.taa_reactive_mask_encoding(),
                gpu.taa_resolve_bind_group_create_count(),
                gpu.take_buffer_uploads(),
                gpu.take_texture_uploads(),
                gpu.take_screen_space_ui_upload_commits(),
                gpu.take_hzb_occlusion_params_commits(),
                gpu.take_history_writes(),
            )
        })
        .unwrap_or_default();
    drop(context);
    if let Some(scope) = gpu_timestamp_scope.as_ref() {
        scope.end(encoder);
    }
    if let Some(error) = surface_present_error {
        return Err(error);
    }
    if let Some(error) = output_target_writeback_error {
        return Err(error);
    }
    execute_result.map_err(GraphicsError::Asset)?;
    let cluster_grid_size = cluster_dimensions_for_size(frame.viewport_size);
    let scene_linear_allocation_size = frame
        .view_family_pipeline()
        .phase_targets(RenderPipelinePhase::SceneLinear)
        .ok_or(GraphicsError::MissingViewFamilyPhase {
            phase: RenderPipelinePhase::SceneLinear,
        })?
        .output()
        .allocation_extent();
    let hzb_plan = HzbBuilder::new(scene_linear_allocation_size).build_plan();
    let hzb_occlusion_indirect_arg_count = mesh_draw_lists
        .map(|lists| lists.occlusion_cull_candidate_arg_count())
        .unwrap_or(0);
    let mut dispatch_context = RenderGraphComputeWorkloadDispatchContext::new(
        [cluster_grid_size.x, cluster_grid_size.y],
        [hzb_plan.hzb_size.x, hzb_plan.hzb_size.y],
        hzb_occlusion_indirect_arg_count,
    );
    if let Some(desc) = pipeline
        .history_epilogue_plan()
        .volumetric_scattering()
        .map(|source| source.desc())
    {
        dispatch_context =
            dispatch_context.with_froxel_grid_size([desc.width, desc.height, desc.depth]);
    }
    if let Some(report) = hzb_occlusion_cull_report {
        dispatch_context =
            dispatch_context.with_indirect_args_dispatch_group_count(report.dispatch_group_count);
    }
    Ok(RecordedGraphPass {
        stage,
        pass_name: pass.name.clone(),
        executor_id: executor_id.as_str().to_string(),
        queue: pass.queue,
        declared_queue: pass.declared_queue,
        dependencies: pass.dependencies.clone(),
        resources: pass.resources.clone(),
        debug_marker: pass_debug_marker,
        budget_key: stage.frame_profile_budget_key(),
        cpu_elapsed_micros,
        render_metrics,
        mesh_replay_stats,
        compute_workload: pass.compute_workload.clone(),
        dispatch_context,
        compute_dispatches,
        motion_vector_camera_status,
        hzb_occlusion_cull_report,
        light_grid_report,
        native_resource_creates,
        taa_reactive_mask_encoding,
        taa_resolve_bind_group_create_count,
        buffer_uploads,
        texture_uploads,
        screen_space_ui_upload_commits,
        hzb_occlusion_params_commits,
        history_writes,
        output_target_writeback_report,
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

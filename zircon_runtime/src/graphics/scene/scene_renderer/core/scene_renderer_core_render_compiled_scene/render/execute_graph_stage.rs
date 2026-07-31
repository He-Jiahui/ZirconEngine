use std::time::{Duration, Instant};

use crate::core::framework::render::{
    MotionVectorCameraStatus, PostProcessGraphResourceNames, PostProcessPassGraph,
    RenderGraphPassProfileMetrics, RenderPluginRendererOutputs,
};
use crate::graphics::backend::{GpuPassTimer, GpuPassTimestampScope};
use crate::graphics::debug_markers::{
    insert_marker, marker_for_render_graph_pass, marker_for_render_pass_stage,
};
use crate::graphics::pipeline::RenderPassStage;
use crate::graphics::pipeline::{CompiledRenderPipeline, CompiledRenderPipelinePassStage};
use crate::graphics::scene::resources::ResourceStreamer;
use crate::graphics::scene::scene_renderer::cluster_dimensions_for_size;
use crate::graphics::scene::scene_renderer::deferred::DeferredSceneResources;
use crate::graphics::scene::scene_renderer::environment::IblBakeWgpuPipelineCache;
use crate::graphics::scene::scene_renderer::graph_execution::{
    RenderGraphComputeWorkloadDispatchContext, RenderGraphExecutionRecord,
    RenderGraphExecutionResources, RenderPassExecutionContext, RenderPassExecutorId,
    RenderPassExecutorRegistry, RenderPassGpuExecutionContext, RenderPassMeshCommandLists,
    RenderPassPostProcessStackContext,
};
use crate::graphics::scene::scene_renderer::hzb::HzbOcclusionCuller;
use crate::graphics::scene::scene_renderer::mesh::MeshPipelineCache;
use crate::graphics::scene::scene_renderer::mesh::mesh_pass::MeshDrawReplayStats;
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
use crate::graphics::visibility::HzbBuilder;

pub(in crate::graphics::scene::scene_renderer::core::scene_renderer_core_render_compiled_scene)
struct RenderGraphStageExecution
<'a> {
    pub(in crate::graphics::scene::scene_renderer::core::scene_renderer_core_render_compiled_scene) resources:
        &'a mut RenderGraphExecutionResources,
    pub(in crate::graphics::scene::scene_renderer::core::scene_renderer_core_render_compiled_scene) record:
        &'a mut RenderGraphExecutionRecord,
    pub(in crate::graphics::scene::scene_renderer::core::scene_renderer_core_render_compiled_scene) plugin_outputs:
        &'a mut RenderPluginRendererOutputs,
    gpu_pass_timer: Option<&'a mut GpuPassTimer>,
}

impl<'a> RenderGraphStageExecution<'a> {
    pub(in crate::graphics::scene::scene_renderer::core::scene_renderer_core_render_compiled_scene) fn new(
        resources: &'a mut RenderGraphExecutionResources,
        record: &'a mut RenderGraphExecutionRecord,
        plugin_outputs: &'a mut RenderPluginRendererOutputs,
        gpu_pass_timer: Option<&'a mut GpuPassTimer>,
    ) -> Self {
        Self {
            resources,
            record,
            plugin_outputs,
            gpu_pass_timer,
        }
    }

    fn begin_gpu_pass(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        pass_name: &str,
    ) -> Option<GpuPassTimestampScope> {
        self.gpu_pass_timer
            .as_deref_mut()
            .and_then(|timer| timer.begin_pass(encoder, pass_name))
    }

    fn end_gpu_pass(&mut self, encoder: &mut wgpu::CommandEncoder, scope: GpuPassTimestampScope) {
        if let Some(timer) = self.gpu_pass_timer.as_deref_mut() {
            timer.end_pass(encoder, scope);
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
}

#[allow(clippy::too_many_arguments)]
pub(in crate::graphics::scene::scene_renderer::core::scene_renderer_core_render_compiled_scene) fn execute_graph_stage(
    pipeline: &CompiledRenderPipeline,
    registry: &RenderPassExecutorRegistry,
    stage: RenderPassStage,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    encoder: &mut wgpu::CommandEncoder,
    frame: &ViewportRenderFrame,
    scene_bind_group_layout: &wgpu::BindGroupLayout,
    target_format: wgpu::TextureFormat,
    depth_format: wgpu::TextureFormat,
    scene_bind_group: &wgpu::BindGroup,
    screen_space_ui_renderer: &mut ScreenSpaceUiRenderer,
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
    execution: &mut RenderGraphStageExecution<'_>,
) -> Result<(), GraphicsError> {
    crate::profile_dynamic_scope!("runtime", "render_graph.stage", format!("{stage:?}"));
    for stage_entry in pipeline
        .pass_stages
        .iter()
        .filter(|entry| entry.stage == stage)
    {
        execute_graph_pass(
            pipeline,
            registry,
            stage_entry,
            device,
            queue,
            encoder,
            frame,
            scene_bind_group_layout,
            target_format,
            depth_format,
            scene_bind_group,
            screen_space_ui_renderer,
            post_process_stack,
            overlay_renderer.as_deref_mut(),
            prepared_overlays,
            deferred,
            particle_renderer,
            sprite_renderer,
            streamer,
            mesh_pipelines.as_deref_mut(),
            ibl_bake_pipeline_cache.as_deref_mut(),
            mesh_draw_lists,
            hzb_occlusion_culler,
            shadow_map_renderer,
            shadow_atlas_resources,
            shadow_frame_plan,
            execution,
        )?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::core::framework::render::{
        PostProcessEffectKind, PostProcessPassGraph, PostProcessPassNode,
        RenderGraphPassProfileMetrics, RenderPluginRendererOutputs,
    };
    use crate::graphics::scene::scene_renderer::graph_execution::{
        RenderGraphExecutionRecord, RenderGraphExecutionResources,
    };
    use crate::graphics::scene::scene_renderer::mesh::mesh_pass::MeshDrawReplayStats;

    use super::{RenderGraphStageExecution, render_profile_metrics_from_mesh_replay_stats};

    #[test]
    fn mesh_replay_counter_delta_maps_to_pass_profile_metrics() {
        let before = MeshDrawReplayStats {
            draw_call_count: 3,
            state_change_count: 5,
            bind_skip_count: 2,
        };
        let after = MeshDrawReplayStats {
            draw_call_count: 7,
            state_change_count: 11,
            bind_skip_count: 4,
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
        let mut execution =
            RenderGraphStageExecution::new(&mut resources, &mut record, &mut plugin_outputs, None);

        execution.record_post_process_graph(&graph);

        assert_eq!(record.post_process_graph(), Some(&graph));
        assert_eq!(
            record.executed_post_process_nodes(),
            &["output-transfer".to_string()]
        );
        assert!(record.executed_passes().is_empty());
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
        assert!(scene_passes.contains(
            "execute_deferred_graph_stage(\n                &self.deferred,\n                &mut self.mesh_pipelines,"
        ));
        assert!(scene_passes.contains(
            "let deferred_lighting_result = execute_deferred_graph_stage(\n                &self.deferred,\n                &mut self.mesh_pipelines,"
        ));
        assert!(
            scene_passes.contains("RenderPassStage::Deferred,\n                Some(streamer),")
        );
        assert!(scene_passes.contains("RenderPassStage::Lighting,\n                None,"));
        assert!(scene_passes.contains("mesh_pipelines: &mut MeshPipelineCache,"));
        assert!(!scene_passes.contains("mesh_pipelines: Option<&mut MeshPipelineCache>,"));
    }
}

#[allow(clippy::too_many_arguments)]
fn execute_graph_pass(
    pipeline: &CompiledRenderPipeline,
    registry: &RenderPassExecutorRegistry,
    stage_entry: &CompiledRenderPipelinePassStage,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    encoder: &mut wgpu::CommandEncoder,
    frame: &ViewportRenderFrame,
    scene_bind_group_layout: &wgpu::BindGroupLayout,
    target_format: wgpu::TextureFormat,
    depth_format: wgpu::TextureFormat,
    scene_bind_group: &wgpu::BindGroup,
    screen_space_ui_renderer: &mut ScreenSpaceUiRenderer,
    post_process_stack: Option<RenderPassPostProcessStackContext<'_>>,
    overlay_renderer: Option<&mut ViewportOverlayRenderer>,
    prepared_overlays: Option<&PreparedOverlayBuffers>,
    deferred: Option<&DeferredSceneResources>,
    particle_renderer: Option<&ParticleRenderer>,
    sprite_renderer: Option<&SpriteRenderer>,
    streamer: Option<&ResourceStreamer>,
    mesh_pipelines: Option<&mut MeshPipelineCache>,
    ibl_bake_pipeline_cache: Option<&mut IblBakeWgpuPipelineCache>,
    mesh_draw_lists: Option<RenderPassMeshCommandLists<'_>>,
    hzb_occlusion_culler: Option<&HzbOcclusionCuller>,
    shadow_map_renderer: Option<&ShadowMapRenderer>,
    shadow_atlas_resources: Option<&ShadowAtlasResources>,
    shadow_frame_plan: Option<&ShadowFramePlan>,
    execution: &mut RenderGraphStageExecution<'_>,
) -> Result<(), GraphicsError> {
    let Some(pass) = pipeline
        .graph()
        .passes()
        .iter()
        .find(|pass| pass.name == stage_entry.pass_name)
    else {
        return Err(GraphicsError::Asset(format!(
            "compiled render pipeline `{}` records stage `{:?}` for missing pass `{}`",
            pipeline.name, stage_entry.stage, stage_entry.pass_name
        )));
    };
    if pass.culled {
        return Ok(());
    }
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
    let gpu_timestamp_scope = execution.begin_gpu_pass(encoder, &pass.name);
    let mut gpu = RenderPassGpuExecutionContext::new(
        device,
        queue,
        encoder,
        frame,
        scene_bind_group_layout,
        target_format,
        depth_format,
        scene_bind_group,
        &mut *execution.resources,
        &mut *execution.plugin_outputs,
        screen_space_ui_renderer,
    );
    gpu.streamer = streamer;
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
    if let Some(ibl_bake_pipeline_cache) = ibl_bake_pipeline_cache {
        gpu = gpu.with_ibl_bake_pipeline_cache(ibl_bake_pipeline_cache);
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
        .with_gpu(gpu);

    let mesh_replay_stats_before = mesh_draw_lists.map(|lists| lists.replay_stats.stats());
    let profile_started = Instant::now();
    let execute_result = registry.execute(&mut context);
    let cpu_elapsed_micros = duration_to_micros(profile_started.elapsed());
    let render_metrics = render_profile_metrics_from_mesh_replay_stats(
        mesh_replay_stats_before,
        mesh_draw_lists.map(|lists| lists.replay_stats.stats()),
    );
    let (
        compute_dispatches,
        motion_vector_camera_status,
        hzb_occlusion_cull_report,
        light_grid_report,
    ) = context
        .gpu_mut()
        .map(|gpu| {
            (
                gpu.take_compute_dispatches(),
                gpu.motion_vector_camera_status(),
                gpu.take_hzb_occlusion_cull_report(),
                gpu.take_light_grid_report(),
            )
        })
        .unwrap_or_default();
    drop(context);
    if let Some(scope) = gpu_timestamp_scope {
        execution.end_gpu_pass(encoder, scope);
    }
    execution
        .record
        .push_pass_profile_with_budget_key_and_compute_dispatches(
            pass.name.clone(),
            executor_id.as_str().to_string(),
            stage_entry.stage.frame_profile_budget_key(),
            cpu_elapsed_micros,
            render_metrics,
            &compute_dispatches,
        );
    execute_result.map_err(GraphicsError::Asset)?;
    let cluster_grid_size = cluster_dimensions_for_size(frame.viewport_size);
    let hzb_plan = HzbBuilder::new(frame.extract.view.effective_render_size()).build_plan();
    let hzb_occlusion_indirect_arg_count = mesh_draw_lists
        .map(|lists| lists.occlusion_cull_candidate_arg_count())
        .unwrap_or(0);
    let mut dispatch_context = RenderGraphComputeWorkloadDispatchContext::new(
        [frame.viewport_size.x, frame.viewport_size.y],
        [cluster_grid_size.x, cluster_grid_size.y],
        [hzb_plan.hzb_size.x, hzb_plan.hzb_size.y],
        hzb_occlusion_indirect_arg_count,
    );
    if let Some(desc) = execution
        .resources
        .owned_texture_desc(PostProcessGraphResourceNames::VOLUMETRIC_SCATTERING)
    {
        dispatch_context =
            dispatch_context.with_froxel_grid_size([desc.width, desc.height, desc.depth]);
    }
    if let Some(report) = hzb_occlusion_cull_report {
        dispatch_context =
            dispatch_context.with_indirect_args_dispatch_group_count(report.dispatch_group_count);
    }
    execution.record.audit_compute_workload(
        &pass.name,
        executor_id.as_str(),
        pass.compute_workload.as_ref(),
        dispatch_context,
        &compute_dispatches,
    );
    if motion_vector_camera_status != MotionVectorCameraStatus::NotRequested {
        execution
            .record
            .set_motion_vector_camera_status(motion_vector_camera_status);
    }
    if let Some(report) = hzb_occlusion_cull_report {
        execution.record.set_hzb_occlusion_cull_report(report);
    }
    if let Some(report) = light_grid_report {
        execution.record.set_light_grid_report(report);
    }
    execution
        .record
        .push_executed_pass_with_stage_declared_queue_dependencies_resources_and_debug_marker(
            Some(stage_entry.stage),
            pass.name.clone(),
            executor_id.as_str().to_string(),
            pass.queue,
            pass.declared_queue,
            pass.dependencies.clone(),
            pass.resources.clone(),
            Some(pass_debug_marker),
        );
    for dispatch in compute_dispatches {
        execution.record.push_compute_dispatch(dispatch);
    }
    Ok(())
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

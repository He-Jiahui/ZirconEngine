use crate::core::framework::render::{
    MotionVectorCameraStatus, PostProcessGraphResourceNames, PostProcessPassGraph,
    RenderPluginRendererOutputs,
};
use crate::graphics::backend::OffscreenTarget;
use crate::graphics::debug_markers::{
    insert_marker, marker_for_render_graph_pass, marker_for_render_pass_stage,
};
use crate::graphics::pipeline::RenderPassStage;
use crate::graphics::pipeline::{CompiledRenderPipeline, CompiledRenderPipelinePassStage};
use crate::graphics::scene::resources::ResourceStreamer;
use crate::graphics::scene::scene_renderer::cluster_dimensions_for_size;
use crate::graphics::scene::scene_renderer::deferred::DeferredSceneResources;
use crate::graphics::scene::scene_renderer::graph_execution::{
    RenderGraphComputeWorkloadDispatchContext, RenderGraphExecutionRecord,
    RenderGraphExecutionResources, RenderGraphImportedFinalTarget, RenderPassExecutionContext,
    RenderPassExecutorId, RenderPassExecutorRegistry, RenderPassGpuExecutionContext,
    RenderPassMeshCommandLists, RenderPassPostProcessStackContext,
};
use crate::graphics::scene::scene_renderer::mesh::MeshPipelineCache;
use crate::graphics::scene::scene_renderer::overlay::{
    PreparedOverlayBuffers, ViewportOverlayRenderer,
};
use crate::graphics::scene::scene_renderer::particle::ParticleRenderer;
use crate::graphics::scene::scene_renderer::post_process::execute_post_process_pass_graph;
use crate::graphics::scene::scene_renderer::prepass::NormalPrepassPipeline;
use crate::graphics::scene::scene_renderer::shadow::ShadowMapRenderer;
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
}

impl<'a> RenderGraphStageExecution<'a> {
    pub(in crate::graphics::scene::scene_renderer::core::scene_renderer_core_render_compiled_scene) fn new(
        resources: &'a mut RenderGraphExecutionResources,
        record: &'a mut RenderGraphExecutionRecord,
        plugin_outputs: &'a mut RenderPluginRendererOutputs,
    ) -> Self {
        Self {
            resources,
            record,
            plugin_outputs,
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

pub(in crate::graphics::scene::scene_renderer::core::scene_renderer_core_render_compiled_scene) fn import_frame_targets(
    resources: &mut RenderGraphExecutionResources,
    target: &OffscreenTarget,
    imported_final_target: Option<RenderGraphImportedFinalTarget<'_>>,
) {
    resources.import_texture_view(
        PostProcessGraphResourceNames::SCENE_COLOR,
        target
            .scene_color
            .create_view(&wgpu::TextureViewDescriptor::default()),
    );
    resources.import_texture_view(
        PostProcessGraphResourceNames::SCENE_DEPTH,
        target
            .depth
            .create_view(&wgpu::TextureViewDescriptor::default()),
    );
    import_final_target_aliases(resources, target, imported_final_target);
    resources.import_texture_view(
        PostProcessGraphResourceNames::GBUFFER_ALBEDO,
        target
            .gbuffer_albedo
            .create_view(&wgpu::TextureViewDescriptor::default()),
    );
    resources.import_texture_view(
        PostProcessGraphResourceNames::GBUFFER_NORMAL,
        target
            .normal
            .create_view(&wgpu::TextureViewDescriptor::default()),
    );
    resources.import_texture_view(
        PostProcessGraphResourceNames::GBUFFER_MATERIAL,
        target
            .gbuffer_material
            .create_view(&wgpu::TextureViewDescriptor::default()),
    );
    resources.import_texture_view(
        PostProcessGraphResourceNames::AMBIENT_OCCLUSION,
        target
            .ambient_occlusion
            .create_view(&wgpu::TextureViewDescriptor::default()),
    );
    resources.import_texture_view(
        PostProcessGraphResourceNames::GLOBAL_ILLUMINATION,
        target
            .global_illumination
            .create_view(&wgpu::TextureViewDescriptor::default()),
    );
    resources.insert_buffer(
        PostProcessGraphResourceNames::LIGHT_LIST,
        target.cluster_buffer.clone(),
    );
    resources.import_texture_view(
        PostProcessGraphResourceNames::BLOOM,
        target
            .bloom
            .create_view(&wgpu::TextureViewDescriptor::default()),
    );
}

fn import_final_target_aliases(
    resources: &mut RenderGraphExecutionResources,
    target: &OffscreenTarget,
    imported_final_target: Option<RenderGraphImportedFinalTarget<'_>>,
) {
    let final_target_aliases = [
        PostProcessGraphResourceNames::FINAL_COLOR,
        PostProcessGraphResourceNames::VIEWPORT_OUTPUT,
        PostProcessGraphResourceNames::FINAL_COMPOSITED,
        PostProcessGraphResourceNames::COLOR_GRADED,
        PostProcessGraphResourceNames::EFFECT_STACKED,
        PostProcessGraphResourceNames::HISTORY_OUTPUT_SCENE_COLOR,
    ];
    if let Some(imported_final_target) = imported_final_target {
        for alias in final_target_aliases {
            resources.import_borrowed_texture_view(alias, imported_final_target.view);
        }
    } else {
        for alias in final_target_aliases {
            resources.import_texture_alias(alias, &target.final_color);
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
    encoder: &mut wgpu::CommandEncoder,
    frame: &ViewportRenderFrame,
    scene_bind_group: &wgpu::BindGroup,
    screen_space_ui_renderer: &mut ScreenSpaceUiRenderer,
    post_process_stack: Option<RenderPassPostProcessStackContext<'_>>,
    mut overlay_renderer: Option<&mut ViewportOverlayRenderer>,
    prepared_overlays: Option<&PreparedOverlayBuffers>,
    prepass: Option<&NormalPrepassPipeline>,
    deferred: Option<&DeferredSceneResources>,
    particle_renderer: Option<&ParticleRenderer>,
    sprite_renderer: Option<&SpriteRenderer>,
    streamer: Option<&ResourceStreamer>,
    mut mesh_pipelines: Option<&mut MeshPipelineCache>,
    mesh_draw_lists: Option<RenderPassMeshCommandLists<'_>>,
    shadow_map_renderer: Option<&ShadowMapRenderer>,
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
            scene_bind_group,
            screen_space_ui_renderer,
            post_process_stack,
            overlay_renderer.as_deref_mut(),
            prepared_overlays,
            prepass,
            deferred,
            particle_renderer,
            sprite_renderer,
            streamer,
            mesh_pipelines.as_deref_mut(),
            mesh_draw_lists,
            shadow_map_renderer,
            execution,
        )?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::core::framework::render::{
        PostProcessEffectKind, PostProcessGraphResourceNames, PostProcessPassGraph,
        PostProcessPassNode, RenderPluginRendererOutputs,
    };
    use crate::core::math::UVec2;
    use crate::graphics::backend::{OffscreenTarget, RenderBackend};
    use crate::graphics::scene::scene_renderer::graph_execution::{
        RenderGraphExecutionRecord, RenderGraphExecutionResources, RenderGraphImportedFinalTarget,
    };
    use crate::scene::world::World;
    use crate::RenderPipelineAsset;

    use super::{import_frame_targets, RenderGraphStageExecution};

    #[test]
    fn stage_execution_records_post_process_graph_through_record_owner() {
        let graph = PostProcessPassGraph {
            nodes: vec![PostProcessPassNode {
                name: "final-composite".to_string(),
                kind: PostProcessEffectKind::FinalComposite,
                required_inputs: Vec::new(),
                produced_outputs: Vec::new(),
                after: Vec::new(),
            }],
            skipped_nodes: Vec::new(),
            final_composite_node: Some("final-composite".to_string()),
        };
        let mut resources = RenderGraphExecutionResources::new();
        let mut record = RenderGraphExecutionRecord::default();
        let mut plugin_outputs = RenderPluginRendererOutputs::default();
        let mut execution =
            RenderGraphStageExecution::new(&mut resources, &mut record, &mut plugin_outputs);

        execution.record_post_process_graph(&graph);

        assert_eq!(record.post_process_graph(), Some(&graph));
        assert_eq!(
            record.executed_post_process_nodes(),
            &["final-composite".to_string()]
        );
        assert!(record.executed_passes().is_empty());
    }

    #[test]
    fn import_frame_targets_binds_persistent_frame_targets_only() {
        let backend = RenderBackend::new_offscreen().unwrap();
        let target = OffscreenTarget::new(&backend.device, UVec2::new(16, 16));
        let mut resources = RenderGraphExecutionResources::new();

        import_frame_targets(&mut resources, &target, None);

        assert!(resources.has_texture_view(PostProcessGraphResourceNames::SCENE_COLOR));
        assert!(resources.has_texture_view(PostProcessGraphResourceNames::SCENE_DEPTH));
        assert!(resources.has_texture_view(PostProcessGraphResourceNames::FINAL_COLOR));
        assert!(resources.has_texture_view(PostProcessGraphResourceNames::VIEWPORT_OUTPUT));
        assert!(resources.has_texture_view(PostProcessGraphResourceNames::GBUFFER_NORMAL));
        assert!(resources.has_texture_view(PostProcessGraphResourceNames::AMBIENT_OCCLUSION));
        assert!(resources.has_texture_view(PostProcessGraphResourceNames::GLOBAL_ILLUMINATION));
        assert!(resources.has_buffer(PostProcessGraphResourceNames::LIGHT_LIST));
        for resource in ADVANCED_POST_PROCESS_TRANSIENTS {
            assert!(
                !resources.has_texture_view(resource),
                "`{resource}` must be graph-owned, not pre-bound to the fixed offscreen target"
            );
        }
    }

    #[test]
    fn import_frame_targets_rebinds_final_aliases_to_imported_texture_target() {
        let backend = RenderBackend::new_offscreen().unwrap();
        let target = OffscreenTarget::new(&backend.device, UVec2::new(16, 16));
        let imported = backend.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("zircon-test-imported-final-target"),
            size: wgpu::Extent3d {
                width: 16,
                height: 16,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let imported_view = imported.create_view(&wgpu::TextureViewDescriptor::default());
        let mut resources = RenderGraphExecutionResources::new();

        import_frame_targets(
            &mut resources,
            &target,
            Some(RenderGraphImportedFinalTarget {
                view: &imported_view,
            }),
        );

        for resource in FINAL_TARGET_ALIASES {
            assert!(
                resources.has_texture_view(resource),
                "`{resource}` should bind to the imported final target"
            );
        }
        assert!(resources.has_texture_view(PostProcessGraphResourceNames::SCENE_COLOR));
        assert!(resources.has_texture_view(PostProcessGraphResourceNames::SCENE_DEPTH));
        let report = resources.resource_report();
        assert!(
            report.external_texture_view_count >= FINAL_TARGET_ALIASES.len(),
            "imported final target aliases should count as external graph views; report={report:?}"
        );
    }

    #[test]
    fn graph_materialization_backs_advanced_post_process_transients() {
        let backend = RenderBackend::new_offscreen().unwrap();
        let target = OffscreenTarget::new(&backend.device, UVec2::new(16, 16));
        let mut extract = World::new().to_render_frame_extract();
        extract.apply_viewport_size(UVec2::new(16, 16));
        let pipeline = RenderPipelineAsset::default_forward_plus()
            .compile(&extract)
            .expect("default forward pipeline should compile for transient ownership test");
        let mut resources = RenderGraphExecutionResources::new();

        import_frame_targets(&mut resources, &target, None);
        resources
            .materialize_transient_resources(&backend.device, &pipeline.graph)
            .expect("advanced post-process transient graph resources should materialize");

        for resource in ADVANCED_POST_PROCESS_TRANSIENTS {
            assert!(
                resources.has_texture_view(resource),
                "`{resource}` should be backed by graph materialization"
            );
        }
        let report = resources.resource_report();
        let aliased_advanced_transient_count = 2;
        assert!(
            report.owned_texture_count
                >= ADVANCED_POST_PROCESS_TRANSIENTS
                    .len()
                    .saturating_sub(aliased_advanced_transient_count),
            "owned texture count should include physically backed advanced post-process transients; report={report:?}"
        );
        assert!(
            report.texture_view_count >= ADVANCED_POST_PROCESS_TRANSIENTS.len(),
            "texture view count should include logical advanced post-process resources; report={report:?}"
        );
    }

    const ADVANCED_POST_PROCESS_TRANSIENTS: &[&str] = &[
        PostProcessGraphResourceNames::SCENE_MOTION_VECTOR,
        PostProcessGraphResourceNames::MOTION_VECTOR_TILE_MAX,
        PostProcessGraphResourceNames::MOTION_VECTOR_TILE_MAX_COARSE,
        PostProcessGraphResourceNames::MOTION_VECTOR_NEIGHBOR_MAX,
        PostProcessGraphResourceNames::DEPTH_OF_FIELD_COC,
        PostProcessGraphResourceNames::DEPTH_OF_FIELD_BOKEH,
        PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_DEPTH_PYRAMID,
        PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_DEPTH_PYRAMID_COARSE,
        PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_REFLECTION_PYRAMID,
        PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_REFLECTION_PYRAMID_COARSE,
        PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_SPECULAR_OCCLUSION,
        PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_HISTORY,
    ];

    const FINAL_TARGET_ALIASES: &[&str] = &[
        PostProcessGraphResourceNames::FINAL_COLOR,
        PostProcessGraphResourceNames::VIEWPORT_OUTPUT,
        PostProcessGraphResourceNames::FINAL_COMPOSITED,
        PostProcessGraphResourceNames::COLOR_GRADED,
        PostProcessGraphResourceNames::EFFECT_STACKED,
        PostProcessGraphResourceNames::HISTORY_OUTPUT_SCENE_COLOR,
    ];
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
    scene_bind_group: &wgpu::BindGroup,
    screen_space_ui_renderer: &mut ScreenSpaceUiRenderer,
    post_process_stack: Option<RenderPassPostProcessStackContext<'_>>,
    overlay_renderer: Option<&mut ViewportOverlayRenderer>,
    prepared_overlays: Option<&PreparedOverlayBuffers>,
    prepass: Option<&NormalPrepassPipeline>,
    deferred: Option<&DeferredSceneResources>,
    particle_renderer: Option<&ParticleRenderer>,
    sprite_renderer: Option<&SpriteRenderer>,
    streamer: Option<&ResourceStreamer>,
    mesh_pipelines: Option<&mut MeshPipelineCache>,
    mesh_draw_lists: Option<RenderPassMeshCommandLists<'_>>,
    shadow_map_renderer: Option<&ShadowMapRenderer>,
    execution: &mut RenderGraphStageExecution<'_>,
) -> Result<(), GraphicsError> {
    let Some(pass) = pipeline
        .graph
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
    let mut gpu = RenderPassGpuExecutionContext::new(
        device,
        queue,
        encoder,
        frame,
        scene_bind_group,
        &mut *execution.resources,
        &mut *execution.plugin_outputs,
        screen_space_ui_renderer,
    );
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
    if let (Some(prepass), Some(mesh_draw_lists)) = (prepass, mesh_draw_lists) {
        gpu = gpu.with_prepass_renderer(prepass, mesh_draw_lists);
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
        gpu = gpu.with_deferred_renderer(deferred, mesh_draw_lists);
    }
    if let Some(particle_renderer) = particle_renderer {
        gpu = gpu.with_particle_renderer(particle_renderer);
    }
    if let (Some(mesh_pipelines), Some(streamer), Some(mesh_draw_lists)) =
        (mesh_pipelines, streamer, mesh_draw_lists)
    {
        gpu = gpu.with_mesh_renderer(mesh_pipelines, streamer, mesh_draw_lists);
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
        .with_resource_resolver(&pipeline.graph, pass.id)
        .with_gpu(gpu);

    registry
        .execute(&mut context)
        .map_err(GraphicsError::Asset)?;
    let (compute_dispatches, motion_vector_camera_status) = context
        .gpu_mut()
        .map(|gpu| {
            (
                gpu.take_compute_dispatches(),
                gpu.motion_vector_camera_status(),
            )
        })
        .unwrap_or_default();
    let cluster_grid_size = cluster_dimensions_for_size(frame.viewport_size);
    let hzb_plan = HzbBuilder::new(frame.extract.view.effective_render_size()).build_plan();
    let dispatch_context = RenderGraphComputeWorkloadDispatchContext::new(
        [frame.viewport_size.x, frame.viewport_size.y],
        [cluster_grid_size.x, cluster_grid_size.y],
        [hzb_plan.hzb_size.x, hzb_plan.hzb_size.y],
    );
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

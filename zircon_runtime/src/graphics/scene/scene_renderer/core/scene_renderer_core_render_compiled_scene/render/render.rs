use crate::core::framework::render::RenderPluginRendererOutputs;
use crate::graphics::backend::OffscreenTarget;
use crate::graphics::debug_markers::{
    insert_marker, pop_group, push_group, RENDERDOC_MARKER_FRAME_EXTRACT,
    RENDERDOC_MARKER_HISTORY_COPY, RENDERDOC_MARKER_POST_PROCESS, RENDERDOC_MARKER_PREPASS,
};
use crate::graphics::pipeline::RenderPassStage;
use crate::graphics::scene::resources::ResourceStreamer;
use crate::graphics::scene::scene_renderer::graph_execution::{
    RenderGraphExecutionRecord, RenderGraphExecutionResources, RenderPassExecutorRegistry,
    RenderPassPostProcessStackContext,
};
use crate::graphics::scene::scene_renderer::history::SceneFrameHistoryTextures;
use crate::graphics::scene::scene_renderer::mesh::prepare_mesh_queue;
use crate::graphics::scene::scene_renderer::post_process::SceneRuntimeFeatureFlags;
use crate::graphics::scene::scene_renderer::sprite::prepare_sprite_queue_stats;
use crate::graphics::types::{GraphicsError, ViewportRenderFrame};
use crate::CompiledRenderPipeline;

use super::super::super::scene_renderer_core::{
    merge_plugin_renderer_outputs, SceneRendererAdvancedPluginReadbacks, SceneRendererCore,
};
use super::super::SceneRendererCompiledSceneOutputs;
use super::assign_execution_owned_indirect_args::assign_execution_owned_indirect_args;
use super::build_compiled_scene_draws::build_compiled_scene_draws;
use super::execute_graph_stage::{
    execute_graph_stage, import_frame_targets, RenderGraphStageExecution,
};
use super::partition_mesh_draws::partition_mesh_draws;
use super::prepare_overlay_buffers::prepare_overlay_buffers;

const EARLY_GRAPH_STAGES: &[RenderPassStage] = &[
    RenderPassStage::DepthPrepass,
    RenderPassStage::Shadow,
    RenderPassStage::AmbientOcclusion,
];

const LATE_GRAPH_STAGES: &[RenderPassStage] = &[
    RenderPassStage::Ui,
    RenderPassStage::Overlay,
    RenderPassStage::Debug,
];

const SPRITE_GRAPH_STAGES: &[RenderPassStage] = &[
    RenderPassStage::Opaque2d,
    RenderPassStage::AlphaMask2d,
    RenderPassStage::Transparent2d,
];

impl SceneRendererCore {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn render_compiled_scene(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        streamer: &ResourceStreamer,
        frame: &ViewportRenderFrame,
        target: &mut OffscreenTarget,
        pipeline: &CompiledRenderPipeline,
        render_pass_executors: &RenderPassExecutorRegistry,
        runtime_features: SceneRuntimeFeatureFlags,
        history_textures: Option<&mut SceneFrameHistoryTextures>,
        history_available: bool,
    ) -> Result<SceneRendererCompiledSceneOutputs, GraphicsError> {
        render_pass_executors
            .validate_compiled_pipeline(pipeline)
            .map_err(GraphicsError::Asset)?;
        self.write_scene_uniform(queue, frame);
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("zircon-compiled-scene-encoder"),
        });
        insert_marker(&mut encoder, RENDERDOC_MARKER_FRAME_EXTRACT);
        let mut compiled_scene_draws = build_compiled_scene_draws(
            &self.advanced_plugin_resources,
            device,
            &mut encoder,
            &self.model_bind_group_layout,
            streamer,
            frame,
            runtime_features.virtual_geometry_enabled,
        );
        let _execution_args_buffer = assign_execution_owned_indirect_args(
            device,
            &mut encoder,
            compiled_scene_draws.draws_mut(),
            runtime_features.deferred_lighting_enabled,
        );
        let mesh_draw_partitions = partition_mesh_draws(compiled_scene_draws.draws());
        let non_transparent_mesh_draws = mesh_draw_partitions.non_transparent();
        let prepared_mesh_queue = prepare_mesh_queue(compiled_scene_draws.draws());
        debug_assert_eq!(
            prepared_mesh_queue.stats().draw_count,
            compiled_scene_draws.draws().len()
        );
        let prepared_sprite_queue_stats = runtime_features
            .sprite_rendering_enabled
            .then(|| prepare_sprite_queue_stats(frame, active_sprite_graph_stages(pipeline)))
            .unwrap_or_default();
        let mesh_draw_lists =
            crate::graphics::scene::scene_renderer::graph_execution::RenderPassMeshDrawLists {
                depth_prepass: prepared_mesh_queue.early_z_draws(),
                opaque: &mesh_draw_partitions.opaque,
                alpha_mask: &mesh_draw_partitions.alpha_mask,
                transparent: &mesh_draw_partitions.transparent,
                non_transparent: &non_transparent_mesh_draws,
            };
        let prepared_overlays = prepare_overlay_buffers(self, device, queue, streamer, frame)?;

        let advanced_plugin_readbacks =
            self.execute_runtime_prepare_passes(device, queue, &mut encoder, streamer, frame)?;
        let mut graph_resources = RenderGraphExecutionResources::new();
        import_frame_targets(&mut graph_resources, target);
        if let Some(history_textures) = history_available
            .then(|| history_textures.as_deref())
            .flatten()
        {
            graph_resources.import_texture_view(
                crate::core::framework::render::PostProcessGraphResourceNames::HISTORY_PREVIOUS_SCENE_COLOR,
                history_textures
                    .scene_color
                    .create_view(&wgpu::TextureViewDescriptor::default()),
            );
        }
        let mut graph_execution_record = RenderGraphExecutionRecord::default();
        let mut graph_plugin_outputs = RenderPluginRendererOutputs::default();
        let mut graph_execution = RenderGraphStageExecution::new(
            &mut graph_resources,
            &mut graph_execution_record,
            &mut graph_plugin_outputs,
        );
        let early_post_process_stack = RenderPassPostProcessStackContext::new(
            &self.post_process,
            &*target,
            streamer,
            runtime_features,
            history_textures.as_deref(),
            history_available,
        );
        for stage in EARLY_GRAPH_STAGES {
            let is_depth_prepass = *stage == RenderPassStage::DepthPrepass;
            if is_depth_prepass {
                push_group(&mut encoder, RENDERDOC_MARKER_PREPASS);
            }
            let overlay_renderer = if is_depth_prepass {
                Some(&mut self.overlay_renderer)
            } else {
                None
            };
            let stage_result = execute_graph_stage(
                pipeline,
                render_pass_executors,
                *stage,
                device,
                queue,
                &mut encoder,
                frame,
                &self.scene_bind_group,
                &mut self.screen_space_ui_renderer,
                Some(early_post_process_stack),
                overlay_renderer,
                None,
                is_depth_prepass.then_some(&self.normal_prepass),
                None,
                None,
                None,
                None,
                None,
                is_depth_prepass.then_some(mesh_draw_lists),
                &mut graph_execution,
            );
            if is_depth_prepass {
                pop_group(&mut encoder);
            }
            stage_result?;
        }
        if !runtime_features.deferred_lighting_enabled {
            execute_graph_stage(
                pipeline,
                render_pass_executors,
                RenderPassStage::Lighting,
                device,
                queue,
                &mut encoder,
                frame,
                &self.scene_bind_group,
                &mut self.screen_space_ui_renderer,
                Some(early_post_process_stack),
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                &mut graph_execution,
            )?;
        }
        self.render_scene_passes(
            device,
            queue,
            &mut encoder,
            streamer,
            frame,
            target,
            runtime_features,
            pipeline,
            render_pass_executors,
            &mut graph_execution,
            mesh_draw_lists,
            history_textures.as_deref(),
            history_available,
        )?;
        let mut runtime_frame = frame.clone();
        if !history_available {
            runtime_frame.extract.post_process.stack = runtime_frame
                .extract
                .post_process
                .stack
                .without_history_resources();
            runtime_frame.extract.post_process.graph =
                runtime_frame.extract.post_process.stack.validated_graph();
        }
        insert_marker(&mut encoder, RENDERDOC_MARKER_POST_PROCESS);
        let post_process_stack = RenderPassPostProcessStackContext::new(
            &self.post_process,
            &*target,
            streamer,
            runtime_features,
            history_textures.as_deref(),
            history_available,
        );
        execute_graph_stage(
            pipeline,
            render_pass_executors,
            RenderPassStage::PostProcess,
            device,
            queue,
            &mut encoder,
            &runtime_frame,
            &self.scene_bind_group,
            &mut self.screen_space_ui_renderer,
            Some(post_process_stack),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            &mut graph_execution,
        )?;
        graph_execution.record_post_process_graph(&runtime_frame.extract.post_process.graph);
        let history_copy_required = history_textures.is_some()
            && (runtime_features.history_resolve_enabled
                || runtime_features.hybrid_global_illumination_enabled
                || runtime_features.ssao_enabled);
        if history_copy_required {
            insert_marker(&mut encoder, RENDERDOC_MARKER_HISTORY_COPY);
        }
        self.copy_history_textures(&mut encoder, target, history_textures, runtime_features);
        for stage in LATE_GRAPH_STAGES {
            let overlay_stage = matches!(*stage, RenderPassStage::Overlay | RenderPassStage::Debug);
            let overlay_renderer = if overlay_stage {
                Some(&mut self.overlay_renderer)
            } else {
                None
            };
            let prepared_overlay_buffers = overlay_stage.then_some(&prepared_overlays);
            execute_graph_stage(
                pipeline,
                render_pass_executors,
                *stage,
                device,
                queue,
                &mut encoder,
                frame,
                &self.scene_bind_group,
                &mut self.screen_space_ui_renderer,
                None,
                overlay_renderer,
                prepared_overlay_buffers,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                &mut graph_execution,
            )?;
        }
        drop(graph_execution);
        queue.submit([encoder.finish()]);
        let mut renderer_outputs = advanced_plugin_readbacks.into_outputs();
        merge_plugin_renderer_outputs(&mut renderer_outputs, graph_plugin_outputs);
        Ok(SceneRendererCompiledSceneOutputs::new(
            SceneRendererAdvancedPluginReadbacks::from_outputs(renderer_outputs),
            graph_execution_record,
            prepared_mesh_queue.stats(),
            prepared_sprite_queue_stats,
        ))
    }
}

fn active_sprite_graph_stages(pipeline: &CompiledRenderPipeline) -> Vec<RenderPassStage> {
    SPRITE_GRAPH_STAGES
        .iter()
        .copied()
        .filter(|stage| pipeline_has_active_sprite_stage(pipeline, *stage))
        .collect()
}

fn pipeline_has_active_sprite_stage(
    pipeline: &CompiledRenderPipeline,
    stage: RenderPassStage,
) -> bool {
    pipeline
        .pass_stages
        .iter()
        .filter(|stage_entry| stage_entry.stage == stage)
        .any(|stage_entry| {
            pipeline.graph.passes().iter().any(|pass| {
                pass.name == stage_entry.pass_name
                    && !pass.culled
                    && pass
                        .executor_id
                        .as_deref()
                        .is_some_and(|executor_id| executor_id.starts_with("sprite."))
            })
        })
}

#[cfg(test)]
mod tests {
    use super::{
        active_sprite_graph_stages, EARLY_GRAPH_STAGES, LATE_GRAPH_STAGES, SPRITE_GRAPH_STAGES,
    };
    use crate::core::framework::render::RenderPipelineHandle;
    use crate::graphics::pipeline::RenderPassStage;
    use crate::graphics::pipeline::{CompiledRenderPipeline, CompiledRenderPipelinePassStage};
    use crate::render_graph::{QueueLane, RenderGraphBuilder};

    #[test]
    fn compiled_scene_graph_stage_lists_cover_core2d_product_stages() {
        assert!(!EARLY_GRAPH_STAGES.contains(&RenderPassStage::Opaque2d));
        assert!(!EARLY_GRAPH_STAGES.contains(&RenderPassStage::AlphaMask2d));
        assert!(!EARLY_GRAPH_STAGES.contains(&RenderPassStage::Transparent2d));
        assert!(!EARLY_GRAPH_STAGES.contains(&RenderPassStage::Deferred));
        assert!(!EARLY_GRAPH_STAGES.contains(&RenderPassStage::Lighting));
        assert!(!EARLY_GRAPH_STAGES.contains(&RenderPassStage::AlphaMask3d));
        assert_eq!(
            LATE_GRAPH_STAGES,
            &[
                RenderPassStage::Ui,
                RenderPassStage::Overlay,
                RenderPassStage::Debug,
            ]
        );
    }

    #[test]
    fn active_sprite_graph_stages_follow_unculled_sprite_passes() {
        let pipeline = compiled_pipeline_with_passes([
            (RenderPassStage::Opaque2d, "sprite-opaque", "sprite.opaque"),
            (
                RenderPassStage::Transparent2d,
                "sprite-transparent",
                "sprite.transparent",
            ),
            (RenderPassStage::Ui, "runtime-ui", "ui.screen-space"),
        ]);

        assert_eq!(
            active_sprite_graph_stages(&pipeline),
            vec![RenderPassStage::Opaque2d, RenderPassStage::Transparent2d]
        );
    }

    fn compiled_pipeline_with_passes<const N: usize>(
        passes: [(RenderPassStage, &str, &str); N],
    ) -> CompiledRenderPipeline {
        let mut graph = RenderGraphBuilder::new("sprite-stage-test");
        let mut pass_stages = Vec::new();
        for (stage, pass_name, executor_id) in passes {
            graph.add_pass_with_executor(pass_name, QueueLane::Graphics, Some(executor_id));
            pass_stages.push(CompiledRenderPipelinePassStage::new(pass_name, stage));
        }

        CompiledRenderPipeline {
            handle: RenderPipelineHandle::new(99),
            name: "sprite-stage-test".to_string(),
            renderer_name: "sprite-stage-test".to_string(),
            stages: SPRITE_GRAPH_STAGES.to_vec(),
            pass_stages,
            enabled_features: Vec::new(),
            required_extract_sections: Vec::new(),
            capability_requirements: Vec::new(),
            history_bindings: Vec::new(),
            graph: graph.compile().expect("sprite stage test graph"),
        }
    }
}

use crate::core::framework::render::RenderPluginRendererOutputs;
use crate::graphics::backend::OffscreenTarget;
use crate::graphics::pipeline::RenderPassStage;
use crate::graphics::scene::scene_renderer::graph_execution::{
    RenderGraphExecutionRecord, RenderGraphExecutionResources, RenderPassExecutorRegistry,
};
use crate::graphics::scene::resources::ResourceStreamer;
use crate::graphics::scene::scene_renderer::history::SceneFrameHistoryTextures;
use crate::graphics::scene::scene_renderer::post_process::SceneRuntimeFeatureFlags;
use crate::graphics::types::{GraphicsError, ViewportRenderFrame};
use crate::CompiledRenderPipeline;

use super::super::super::scene_renderer_core::SceneRendererCore;
use super::super::super::scene_renderer_core::SceneRendererAdvancedPluginReadbacks;
use super::super::SceneRendererCompiledSceneOutputs;
use super::assign_execution_owned_indirect_args::assign_execution_owned_indirect_args;
use super::build_compiled_scene_draws::build_compiled_scene_draws;
use super::execute_graph_stage::{
    execute_graph_stage, import_frame_targets, RenderGraphStageExecution,
};
use super::partition_mesh_draws::partition_mesh_draws;
use super::prepare_overlay_buffers::prepare_overlay_buffers;

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
        let (opaque_mesh_draws, transparent_mesh_draws) =
            partition_mesh_draws(compiled_scene_draws.draws());
        let _execution_draws = if runtime_features.deferred_lighting_enabled {
            opaque_mesh_draws
                .iter()
                .copied()
                .chain(transparent_mesh_draws.iter().copied())
                .collect::<Vec<_>>()
        } else {
            compiled_scene_draws.draws().iter().collect::<Vec<_>>()
        };
        let prepared_overlays = prepare_overlay_buffers(self, device, queue, streamer, frame)?;

        let advanced_plugin_readbacks =
            self.execute_runtime_prepare_passes(device, queue, &mut encoder, streamer, frame)?;
        let mut graph_resources = RenderGraphExecutionResources::new();
        import_frame_targets(&mut graph_resources, target);
        let mut graph_execution_record = RenderGraphExecutionRecord::default();
        let mut graph_plugin_outputs = RenderPluginRendererOutputs::default();
        let mut graph_execution = RenderGraphStageExecution::new(
            &mut graph_resources,
            &mut graph_execution_record,
            &mut graph_plugin_outputs,
        );
        for stage in [
            RenderPassStage::DepthPrepass,
            RenderPassStage::Shadow,
            RenderPassStage::GBuffer,
            RenderPassStage::AmbientOcclusion,
            RenderPassStage::Lighting,
            RenderPassStage::Opaque,
        ] {
            execute_graph_stage(
                pipeline,
                render_pass_executors,
                stage,
                device,
                queue,
                &mut encoder,
                frame,
                &self.scene_bind_group,
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
            compiled_scene_draws.draws(),
            &opaque_mesh_draws,
            &transparent_mesh_draws,
        )?;
        execute_graph_stage(
            pipeline,
            render_pass_executors,
            RenderPassStage::PostProcess,
            device,
            queue,
            &mut encoder,
            frame,
            &self.scene_bind_group,
            &mut graph_execution,
        )?;
        self.execute_post_process_stack(
            device,
            queue,
            &mut encoder,
            target,
            frame,
            runtime_features,
            None,
            history_textures.as_deref(),
            history_available,
        );
        self.copy_history_textures(&mut encoder, target, history_textures, runtime_features);
        execute_graph_stage(
            pipeline,
            render_pass_executors,
            RenderPassStage::Overlay,
            device,
            queue,
            &mut encoder,
            frame,
            &self.scene_bind_group,
            &mut graph_execution,
        )?;
        self.overlay_renderer.record_overlays(
            &mut encoder,
            &target.final_color_view,
            &target.depth_view,
            &self.scene_bind_group,
            frame,
            &prepared_overlays,
        );
        self.screen_space_ui_renderer.record(
            device,
            queue,
            &mut encoder,
            &target.final_color_view,
            frame,
        );

        queue.submit([encoder.finish()]);
        let mut renderer_outputs = advanced_plugin_readbacks.into_outputs();
        merge_plugin_renderer_outputs(&mut renderer_outputs, graph_plugin_outputs);
        Ok(SceneRendererCompiledSceneOutputs::new(
            SceneRendererAdvancedPluginReadbacks::from_outputs(renderer_outputs),
            graph_execution_record,
        ))
    }
}

fn merge_plugin_renderer_outputs(
    base: &mut RenderPluginRendererOutputs,
    graph: RenderPluginRendererOutputs,
) {
    if !graph.virtual_geometry.is_empty() {
        base.virtual_geometry = graph.virtual_geometry;
    }
    if !graph.hybrid_gi.is_empty() {
        base.hybrid_gi = graph.hybrid_gi;
    }
    if !graph.particles.is_empty() {
        base.particles = graph.particles;
    }
}

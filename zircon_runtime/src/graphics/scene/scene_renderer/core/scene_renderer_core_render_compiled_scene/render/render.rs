use crate::core::framework::render::RenderPluginRendererOutputs;
use crate::graphics::backend::OffscreenTarget;
use crate::graphics::debug_markers::{
    insert_marker, RENDERDOC_MARKER_FRAME_EXTRACT, RENDERDOC_MARKER_HISTORY_COPY,
    RENDERDOC_MARKER_OVERLAY, RENDERDOC_MARKER_POST_PROCESS,
};
use crate::graphics::pipeline::RenderPassStage;
use crate::graphics::scene::resources::ResourceStreamer;
use crate::graphics::scene::scene_renderer::graph_execution::{
    RenderGraphExecutionRecord, RenderGraphExecutionResources, RenderPassExecutorRegistry,
};
use crate::graphics::scene::scene_renderer::history::SceneFrameHistoryTextures;
use crate::graphics::scene::scene_renderer::post_process::{
    build_post_process_pass_graph, execute_post_process_pass_graph, SceneRuntimeFeatureFlags,
};
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
    RenderPassStage::Deferred,
    RenderPassStage::AmbientOcclusion,
    RenderPassStage::Lighting,
    RenderPassStage::Opaque2d,
    RenderPassStage::AlphaMask2d,
    RenderPassStage::Transparent2d,
    RenderPassStage::Opaque3d,
];

const LATE_GRAPH_STAGES: &[RenderPassStage] = &[
    RenderPassStage::Ui,
    RenderPassStage::Overlay,
    RenderPassStage::Debug,
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
        if let Some(history_textures) = history_available
            .then(|| history_textures.as_deref())
            .flatten()
        {
            graph_resources.import_texture_view(
                crate::core::framework::render::PostProcessGraphResourceNames::HISTORY_COLOR,
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
        for stage in EARLY_GRAPH_STAGES {
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
            &mut graph_execution,
        )?;
        let post_process_graph =
            build_post_process_pass_graph(&runtime_frame.extract.post_process.graph);
        execute_post_process_pass_graph(
            &post_process_graph,
            &*graph_execution.resources,
            &mut *graph_execution.record,
        );
        graph_execution
            .record
            .set_post_process_graph(post_process_graph);
        self.execute_post_process_stack(
            device,
            queue,
            &mut encoder,
            target,
            &runtime_frame,
            runtime_features,
            None,
            history_textures.as_deref(),
            history_available,
        );
        let history_copy_required = history_textures.is_some()
            && (runtime_features.history_resolve_enabled
                || runtime_features.hybrid_global_illumination_enabled
                || runtime_features.ssao_enabled);
        if history_copy_required {
            insert_marker(&mut encoder, RENDERDOC_MARKER_HISTORY_COPY);
        }
        self.copy_history_textures(&mut encoder, target, history_textures, runtime_features);
        for stage in LATE_GRAPH_STAGES {
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
                &mut graph_execution,
            )?;
        }
        insert_marker(&mut encoder, RENDERDOC_MARKER_OVERLAY);
        self.overlay_renderer.record_overlays(
            &mut encoder,
            &target.final_color_view,
            &target.depth_view,
            &self.scene_bind_group,
            frame,
            &prepared_overlays,
        );
        drop(graph_execution);
        queue.submit([encoder.finish()]);
        let mut renderer_outputs = advanced_plugin_readbacks.into_outputs();
        merge_plugin_renderer_outputs(&mut renderer_outputs, graph_plugin_outputs);
        Ok(SceneRendererCompiledSceneOutputs::new(
            SceneRendererAdvancedPluginReadbacks::from_outputs(renderer_outputs),
            graph_execution_record,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::{EARLY_GRAPH_STAGES, LATE_GRAPH_STAGES};
    use crate::graphics::pipeline::RenderPassStage;

    #[test]
    fn compiled_scene_graph_stage_lists_cover_core2d_product_stages() {
        assert!(EARLY_GRAPH_STAGES.contains(&RenderPassStage::Transparent2d));
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
}

use crate::core::framework::render::FrameHistoryHandle;

use crate::graphics::types::{GraphicsError, ViewportFrame, ViewportRenderFrame};
use crate::render_graph::QueueLane;
use crate::CompiledRenderPipeline;

use super::super::super::graph_execution::{
    RenderGraphExecutionRecord, RenderPassExecutionContext, RenderPassExecutorId,
    RenderPassExecutorRegistry,
};
use super::super::runtime_features::runtime_features_from_pipeline;
use super::super::scene_renderer::SceneRenderer;
use super::super::scene_renderer_history::prepare_history_textures;
use super::super::scene_renderer_runtime_outputs::{
    reset_last_runtime_outputs, store_last_runtime_outputs,
};
use super::super::scene_renderer_target::{ensure_offscreen_target, finish_viewport_frame};
use super::super::target_extent::viewport_size;

impl SceneRenderer {
    pub(crate) fn render_frame_with_pipeline(
        &mut self,
        frame: &ViewportRenderFrame,
        pipeline: &CompiledRenderPipeline,
        history_handle: Option<FrameHistoryHandle>,
    ) -> Result<ViewportFrame, GraphicsError> {
        reset_last_runtime_outputs(self);

        self.streamer.ensure_scene_resources(
            &self.backend.device,
            &self.backend.queue,
            &self.core.texture_bind_group_layout,
            frame,
        )?;

        let size = viewport_size(frame);
        ensure_offscreen_target(&self.backend.device, &mut self.target, size);
        let runtime_features = runtime_features_from_pipeline(pipeline);
        self.last_render_graph_execution =
            execute_compiled_graph_passes(pipeline, &self.render_pass_executors)?;

        let runtime_outputs = {
            let (history_textures, history_available) = prepare_history_textures(
                &self.backend.device,
                &self.backend.queue,
                &mut self.history_targets,
                history_handle,
                size,
                runtime_features,
            );
            let target = self.target.as_mut().expect("offscreen target");
            self.core.render_compiled_scene(
                &self.backend.device,
                &self.backend.queue,
                &self.streamer,
                frame,
                target,
                runtime_features,
                history_textures,
                history_available,
            )?
        };

        store_last_runtime_outputs(self, runtime_outputs)?;
        self.generation += 1;

        let target = self.target.as_ref().expect("offscreen target");
        finish_viewport_frame(
            &self.backend.device,
            &self.backend.queue,
            target,
            self.generation,
        )
    }
}

impl SceneRenderer {
    pub(crate) fn validate_compiled_pipeline_executors(
        &self,
        pipeline: &CompiledRenderPipeline,
    ) -> Result<(), String> {
        self.render_pass_executors
            .validate_compiled_pipeline(pipeline)
    }

    pub(crate) fn last_render_graph_executed_passes(&self) -> &[String] {
        self.last_render_graph_execution.executed_passes()
    }

    pub(crate) fn last_render_graph_executed_executor_ids(&self) -> &[String] {
        self.last_render_graph_execution.executed_executor_ids()
    }

    pub(crate) fn last_render_graph_executed_resource_access_count(&self) -> usize {
        self.last_render_graph_execution
            .executed_resource_access_count()
    }

    pub(crate) fn last_render_graph_executed_dependency_count(&self) -> usize {
        self.last_render_graph_execution.executed_dependency_count()
    }

    pub(crate) fn last_render_graph_executed_queue_fallback_count(&self) -> usize {
        self.last_render_graph_execution
            .executed_queue_fallback_count()
    }

    pub(crate) fn last_render_graph_executed_queue_lane_count(&self, queue: QueueLane) -> usize {
        self.last_render_graph_execution
            .executed_queue_lane_count(queue)
    }
}

fn execute_compiled_graph_passes(
    pipeline: &CompiledRenderPipeline,
    registry: &RenderPassExecutorRegistry,
) -> Result<RenderGraphExecutionRecord, GraphicsError> {
    registry
        .validate_compiled_pipeline(pipeline)
        .map_err(GraphicsError::Asset)?;

    let mut record = RenderGraphExecutionRecord::default();
    for pass in pipeline.graph.passes().iter().filter(|pass| !pass.culled) {
        let executor_id = pass
            .executor_id
            .as_ref()
            .expect("compiled executable pass should be validated with an executor id");
        let executor_id = RenderPassExecutorId::new(executor_id.clone());
        let context =
            RenderPassExecutionContext::with_declared_graph_metadata_dependencies_and_resources(
                pass.name.clone(),
                executor_id.clone(),
                pass.queue,
                pass.declared_queue,
                pass.flags,
                pass.dependencies.clone(),
                pass.resources.clone(),
            );
        registry.execute(&context).map_err(GraphicsError::Asset)?;
        record.push_executed_pass_with_declared_queue_dependencies_and_resources(
            pass.name.clone(),
            executor_id.as_str().to_string(),
            pass.queue,
            pass.declared_queue,
            pass.dependencies.clone(),
            pass.resources.clone(),
        );
    }
    Ok(record)
}

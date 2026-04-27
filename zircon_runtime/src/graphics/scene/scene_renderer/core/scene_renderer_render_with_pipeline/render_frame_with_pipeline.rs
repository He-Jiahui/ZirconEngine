use crate::core::framework::render::{
    FrameHistoryHandle, RenderVirtualGeometryCullInputSnapshot, RenderVirtualGeometryExtract,
};

use crate::graphics::types::{GraphicsError, ViewportFrame, ViewportRenderFrame};
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
        let previous_node_and_cluster_cull_global_state = self
            .advanced_plugin_outputs
            .previous_virtual_geometry_node_and_cluster_cull_global_state();

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
        let virtual_geometry_cull_input = resolve_virtual_geometry_cull_input(frame);
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
                virtual_geometry_cull_input.as_ref(),
                previous_node_and_cluster_cull_global_state.as_ref(),
                target,
                runtime_features,
                history_textures,
                history_available,
            )?
        };

        store_last_runtime_outputs(
            self,
            runtime_outputs,
            frame.virtual_geometry_debug_snapshot.clone(),
            virtual_geometry_cull_input,
            frame.virtual_geometry_cluster_selection_input_source(),
            frame.extract.geometry.virtual_geometry.as_ref(),
        )?;
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
}

fn resolve_virtual_geometry_cull_input(
    frame: &ViewportRenderFrame,
) -> Option<RenderVirtualGeometryCullInputSnapshot> {
    frame
        .virtual_geometry_debug_snapshot
        .as_ref()
        .map(|snapshot| snapshot.cull_input)
        .or_else(|| {
            let extract = frame.extract.geometry.virtual_geometry.as_ref()?;
            Some(RenderVirtualGeometryCullInputSnapshot {
                cluster_budget: extract.cluster_budget,
                page_budget: extract.page_budget,
                instance_count: saturated_u32_len(extract.instances.len()),
                cluster_count: saturated_u32_len(extract.clusters.len()),
                page_count: saturated_u32_len(extract.pages.len()),
                visible_entity_count: frame
                    .virtual_geometry_prepare
                    .as_ref()
                    .map(|prepare| saturated_u32_len(prepare.visible_entities.len()))
                    .unwrap_or_else(|| unique_extract_entity_count(extract)),
                visible_cluster_count: frame
                    .virtual_geometry_prepare
                    .as_ref()
                    .map(|prepare| saturated_u32_len(prepare.visible_clusters.len()))
                    .unwrap_or_else(|| saturated_u32_len(extract.clusters.len())),
                resident_page_count: frame
                    .virtual_geometry_prepare
                    .as_ref()
                    .map(|prepare| saturated_u32_len(prepare.resident_pages.len()))
                    .unwrap_or(0),
                pending_page_request_count: frame
                    .virtual_geometry_prepare
                    .as_ref()
                    .map(|prepare| saturated_u32_len(prepare.pending_page_requests.len()))
                    .unwrap_or(0),
                available_page_slot_count: frame
                    .virtual_geometry_prepare
                    .as_ref()
                    .map(|prepare| saturated_u32_len(prepare.available_slots.len()))
                    .unwrap_or(0),
                evictable_page_count: frame
                    .virtual_geometry_prepare
                    .as_ref()
                    .map(|prepare| saturated_u32_len(prepare.evictable_pages.len()))
                    .unwrap_or(0),
                debug: extract.debug,
                cluster_selection_input_source: frame
                    .virtual_geometry_cluster_selection_input_source(),
            })
        })
}

fn unique_extract_entity_count(extract: &RenderVirtualGeometryExtract) -> u32 {
    if !extract.instances.is_empty() {
        return saturated_u32_len(
            extract
                .instances
                .iter()
                .map(|instance| instance.entity)
                .collect::<std::collections::BTreeSet<_>>()
                .len(),
        );
    }

    saturated_u32_len(
        extract
            .clusters
            .iter()
            .map(|cluster| cluster.entity)
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
    )
}

fn saturated_u32_len(len: usize) -> u32 {
    u32::try_from(len).unwrap_or(u32::MAX)
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
        let Some(executor_id) = pass.executor_id.as_ref() else {
            continue;
        };
        let executor_id = RenderPassExecutorId::new(executor_id.clone());
        let context = RenderPassExecutionContext::new(pass.name.clone(), executor_id.clone());
        registry.execute(&context).map_err(GraphicsError::Asset)?;
        record.push_executed_pass(pass.name.clone(), executor_id.as_str().to_string());
    }
    Ok(record)
}

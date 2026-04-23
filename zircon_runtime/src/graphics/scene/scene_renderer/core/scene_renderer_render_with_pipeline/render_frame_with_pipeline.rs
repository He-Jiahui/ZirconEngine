use crate::core::framework::render::{
    FrameHistoryHandle, RenderVirtualGeometryCullInputSnapshot, RenderVirtualGeometryExtract,
};

use crate::graphics::types::{GraphicsError, ViewportFrame, ViewportRenderFrame};
use crate::CompiledRenderPipeline;

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
        let virtual_geometry_cull_input = resolve_virtual_geometry_cull_input(frame);

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
                target,
                runtime_features,
                history_textures,
                history_available,
            )?
        };

        let (
            hybrid_gi_gpu_readback,
            virtual_geometry_gpu_readback,
            indirect_draw_count,
            indirect_buffer_count,
            indirect_segment_count,
            execution_segment_count,
            execution_page_count,
            execution_resident_segment_count,
            execution_pending_segment_count,
            execution_missing_segment_count,
            execution_repeated_draw_count,
            execution_indirect_offsets,
            execution_segments,
            executed_selected_clusters,
            executed_selected_cluster_source,
            executed_selected_cluster_count,
            executed_selected_cluster_buffer,
            node_and_cluster_cull_source,
            node_and_cluster_cull_record_count,
            node_and_cluster_cull_global_state,
            node_and_cluster_cull_dispatch_setup,
            node_and_cluster_cull_instance_seeds,
            node_and_cluster_cull_buffer,
            node_and_cluster_cull_dispatch_setup_buffer,
            node_and_cluster_cull_instance_seed_count,
            node_and_cluster_cull_instance_seed_buffer,
            hardware_rasterization_records,
            hardware_rasterization_source,
            hardware_rasterization_record_count,
            hardware_rasterization_buffer,
            visbuffer64_clear_value,
            visbuffer64_entries,
            visbuffer64_source,
            visbuffer64_entry_count,
            visbuffer64_buffer,
            indirect_draw_submission_order,
            indirect_draw_submission_records,
            indirect_draw_submission_token_records,
            indirect_args_buffer,
            indirect_args_count,
            indirect_submission_buffer,
            indirect_authority_buffer,
            indirect_draw_ref_buffer,
            indirect_segment_buffer,
            indirect_execution_submission_buffer,
            indirect_execution_args_buffer,
            indirect_execution_authority_buffer,
        ) = runtime_outputs;
        store_last_runtime_outputs(
            self,
            hybrid_gi_gpu_readback,
            virtual_geometry_gpu_readback,
            frame.virtual_geometry_debug_snapshot.clone(),
            virtual_geometry_cull_input,
            frame.virtual_geometry_cluster_selection_input_source(),
            frame.extract.geometry.virtual_geometry.as_ref(),
            indirect_draw_count,
            indirect_buffer_count,
            indirect_segment_count,
            execution_segment_count,
            execution_page_count,
            execution_resident_segment_count,
            execution_pending_segment_count,
            execution_missing_segment_count,
            execution_repeated_draw_count,
            execution_indirect_offsets,
            execution_segments,
            executed_selected_clusters,
            executed_selected_cluster_source,
            executed_selected_cluster_count,
            executed_selected_cluster_buffer,
            node_and_cluster_cull_source,
            node_and_cluster_cull_record_count,
            node_and_cluster_cull_global_state,
            node_and_cluster_cull_dispatch_setup,
            node_and_cluster_cull_instance_seeds,
            node_and_cluster_cull_buffer,
            node_and_cluster_cull_dispatch_setup_buffer,
            node_and_cluster_cull_instance_seed_count,
            node_and_cluster_cull_instance_seed_buffer,
            hardware_rasterization_records,
            hardware_rasterization_source,
            hardware_rasterization_record_count,
            hardware_rasterization_buffer,
            visbuffer64_clear_value,
            visbuffer64_entries,
            visbuffer64_source,
            visbuffer64_entry_count,
            visbuffer64_buffer,
            indirect_draw_submission_order,
            indirect_draw_submission_records,
            indirect_draw_submission_token_records,
            indirect_args_buffer,
            indirect_args_count,
            indirect_submission_buffer,
            indirect_authority_buffer,
            indirect_draw_ref_buffer,
            indirect_segment_buffer,
            indirect_execution_submission_buffer,
            indirect_execution_args_buffer,
            indirect_execution_authority_buffer,
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

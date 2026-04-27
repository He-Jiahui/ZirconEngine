use std::sync::Arc;

use crate::core::framework::render::{
    RenderVirtualGeometryClusterSelectionInputSource, RenderVirtualGeometryDebugSnapshot,
    RenderVirtualGeometryHardwareRasterizationSource,
    RenderVirtualGeometryNodeAndClusterCullGlobalStateSnapshot,
    RenderVirtualGeometryNodeAndClusterCullSource, RenderVirtualGeometrySelectedClusterSource,
    RenderVirtualGeometryVisBuffer64Source,
};
use crate::graphics::scene::scene_renderer::{HybridGiGpuReadback, VirtualGeometryGpuReadback};

#[derive(Default)]
pub(in crate::graphics::scene::scene_renderer::core) struct SceneRendererAdvancedPluginOutputs {
    pub(in crate::graphics::scene::scene_renderer::core) hybrid_gi_gpu_readback:
        Option<HybridGiGpuReadback>,
    pub(in crate::graphics::scene::scene_renderer::core) virtual_geometry_gpu_readback:
        Option<VirtualGeometryGpuReadback>,
    pub(in crate::graphics::scene::scene_renderer::core) virtual_geometry_debug_snapshot:
        Option<RenderVirtualGeometryDebugSnapshot>,
    pub(in crate::graphics::scene::scene_renderer::core) virtual_geometry_cluster_selection_input_source:
        RenderVirtualGeometryClusterSelectionInputSource,
    pub(in crate::graphics::scene::scene_renderer::core) virtual_geometry_cull_input_buffer:
        Option<Arc<wgpu::Buffer>>,
    pub(in crate::graphics::scene::scene_renderer::core) virtual_geometry_node_and_cluster_cull_source:
        RenderVirtualGeometryNodeAndClusterCullSource,
    pub(in crate::graphics::scene::scene_renderer::core) virtual_geometry_node_and_cluster_cull_record_count:
        u32,
    pub(in crate::graphics::scene::scene_renderer::core) virtual_geometry_node_and_cluster_cull_global_state:
        Option<RenderVirtualGeometryNodeAndClusterCullGlobalStateSnapshot>,
    pub(in crate::graphics::scene::scene_renderer::core) virtual_geometry_node_and_cluster_cull_dispatch_group_count:
        [u32; 3],
    pub(in crate::graphics::scene::scene_renderer::core) virtual_geometry_node_and_cluster_cull_buffer:
        Option<Arc<wgpu::Buffer>>,
    pub(in crate::graphics::scene::scene_renderer::core) virtual_geometry_node_and_cluster_cull_dispatch_setup_buffer:
        Option<Arc<wgpu::Buffer>>,
    pub(in crate::graphics::scene::scene_renderer::core) virtual_geometry_node_and_cluster_cull_launch_worklist_buffer:
        Option<Arc<wgpu::Buffer>>,
    pub(in crate::graphics::scene::scene_renderer::core) virtual_geometry_node_and_cluster_cull_instance_seed_count:
        u32,
    pub(in crate::graphics::scene::scene_renderer::core) virtual_geometry_node_and_cluster_cull_instance_seed_buffer:
        Option<Arc<wgpu::Buffer>>,
    pub(in crate::graphics::scene::scene_renderer::core) virtual_geometry_node_and_cluster_cull_instance_work_item_count:
        u32,
    pub(in crate::graphics::scene::scene_renderer::core) virtual_geometry_node_and_cluster_cull_instance_work_item_buffer:
        Option<Arc<wgpu::Buffer>>,
    pub(in crate::graphics::scene::scene_renderer::core) virtual_geometry_node_and_cluster_cull_cluster_work_item_count:
        u32,
    pub(in crate::graphics::scene::scene_renderer::core) virtual_geometry_node_and_cluster_cull_cluster_work_item_buffer:
        Option<Arc<wgpu::Buffer>>,
    pub(in crate::graphics::scene::scene_renderer::core) virtual_geometry_node_and_cluster_cull_hierarchy_child_id_count:
        u32,
    pub(in crate::graphics::scene::scene_renderer::core) virtual_geometry_node_and_cluster_cull_hierarchy_child_id_buffer:
        Option<Arc<wgpu::Buffer>>,
    pub(in crate::graphics::scene::scene_renderer::core) virtual_geometry_node_and_cluster_cull_child_work_item_count:
        u32,
    pub(in crate::graphics::scene::scene_renderer::core) virtual_geometry_node_and_cluster_cull_child_work_item_buffer:
        Option<Arc<wgpu::Buffer>>,
    pub(in crate::graphics::scene::scene_renderer::core) virtual_geometry_node_and_cluster_cull_traversal_record_count:
        u32,
    pub(in crate::graphics::scene::scene_renderer::core) virtual_geometry_node_and_cluster_cull_traversal_record_buffer:
        Option<Arc<wgpu::Buffer>>,
    pub(in crate::graphics::scene::scene_renderer::core) virtual_geometry_node_and_cluster_cull_page_request_count:
        u32,
    pub(in crate::graphics::scene::scene_renderer::core) virtual_geometry_node_and_cluster_cull_page_request_ids:
        Vec<u32>,
    pub(in crate::graphics::scene::scene_renderer::core) virtual_geometry_node_and_cluster_cull_page_request_buffer:
        Option<Arc<wgpu::Buffer>>,
    pub(in crate::graphics::scene::scene_renderer::core) virtual_geometry_selected_cluster_source:
        RenderVirtualGeometrySelectedClusterSource,
    pub(in crate::graphics::scene::scene_renderer::core) virtual_geometry_selected_cluster_count:
        u32,
    pub(in crate::graphics::scene::scene_renderer::core) virtual_geometry_selected_cluster_buffer:
        Option<Arc<wgpu::Buffer>>,
    pub(in crate::graphics::scene::scene_renderer::core) virtual_geometry_visbuffer64_clear_value:
        u64,
    pub(in crate::graphics::scene::scene_renderer::core) virtual_geometry_visbuffer64_source:
        RenderVirtualGeometryVisBuffer64Source,
    pub(in crate::graphics::scene::scene_renderer::core) virtual_geometry_visbuffer64_entry_count:
        u32,
    pub(in crate::graphics::scene::scene_renderer::core) virtual_geometry_visbuffer64_buffer:
        Option<Arc<wgpu::Buffer>>,
    pub(in crate::graphics::scene::scene_renderer::core) virtual_geometry_hardware_rasterization_record_count:
        u32,
    pub(in crate::graphics::scene::scene_renderer::core) virtual_geometry_hardware_rasterization_source:
        RenderVirtualGeometryHardwareRasterizationSource,
    pub(in crate::graphics::scene::scene_renderer::core) virtual_geometry_hardware_rasterization_buffer:
        Option<Arc<wgpu::Buffer>>,
    pub(in crate::graphics::scene::scene_renderer::core) virtual_geometry_indirect_draw_count: u32,
    pub(in crate::graphics::scene::scene_renderer::core) virtual_geometry_indirect_buffer_count:
        u32,
    pub(in crate::graphics::scene::scene_renderer::core) virtual_geometry_indirect_segment_count:
        u32,
    pub(in crate::graphics::scene::scene_renderer::core) virtual_geometry_execution_segment_count:
        u32,
    pub(in crate::graphics::scene::scene_renderer::core) virtual_geometry_execution_page_count: u32,
    pub(in crate::graphics::scene::scene_renderer::core) virtual_geometry_execution_resident_segment_count:
        u32,
    pub(in crate::graphics::scene::scene_renderer::core) virtual_geometry_execution_pending_segment_count:
        u32,
    pub(in crate::graphics::scene::scene_renderer::core) virtual_geometry_execution_missing_segment_count:
        u32,
    pub(in crate::graphics::scene::scene_renderer::core) virtual_geometry_execution_repeated_draw_count:
        u32,
    pub(in crate::graphics::scene::scene_renderer::core) virtual_geometry_execution_indirect_offsets:
        Vec<u64>,
    pub(in crate::graphics::scene::scene_renderer::core) virtual_geometry_mesh_draw_submission_order:
        Vec<(Option<u32>, u64, u32)>,
    pub(in crate::graphics::scene::scene_renderer::core) virtual_geometry_mesh_draw_submission_records:
        Vec<(u64, u32, u32, usize)>,
    pub(in crate::graphics::scene::scene_renderer::core) virtual_geometry_mesh_draw_submission_token_records:
        Vec<(u64, u32, u32, u32, usize)>,
    pub(in crate::graphics::scene::scene_renderer::core) virtual_geometry_indirect_args_buffer:
        Option<Arc<wgpu::Buffer>>,
    pub(in crate::graphics::scene::scene_renderer::core) virtual_geometry_indirect_args_count: u32,
    pub(in crate::graphics::scene::scene_renderer::core) virtual_geometry_indirect_submission_buffer:
        Option<Arc<wgpu::Buffer>>,
    pub(in crate::graphics::scene::scene_renderer::core) virtual_geometry_indirect_authority_buffer:
        Option<Arc<wgpu::Buffer>>,
    pub(in crate::graphics::scene::scene_renderer::core) virtual_geometry_indirect_draw_refs_buffer:
        Option<Arc<wgpu::Buffer>>,
    pub(in crate::graphics::scene::scene_renderer::core) virtual_geometry_indirect_segments_buffer:
        Option<Arc<wgpu::Buffer>>,
    pub(in crate::graphics::scene::scene_renderer::core) virtual_geometry_indirect_execution_submission_buffer:
        Option<Arc<wgpu::Buffer>>,
    pub(in crate::graphics::scene::scene_renderer::core) virtual_geometry_indirect_execution_args_buffer:
        Option<Arc<wgpu::Buffer>>,
    pub(in crate::graphics::scene::scene_renderer::core) virtual_geometry_indirect_execution_authority_buffer:
        Option<Arc<wgpu::Buffer>>,
}

pub(in crate::graphics::scene::scene_renderer::core) struct VirtualGeometryCullOutputUpdate {
    pub(in crate::graphics::scene::scene_renderer::core) cluster_selection_input_source:
        RenderVirtualGeometryClusterSelectionInputSource,
    pub(in crate::graphics::scene::scene_renderer::core) cull_input_buffer:
        Option<Arc<wgpu::Buffer>>,
    pub(in crate::graphics::scene::scene_renderer::core) node_and_cluster_cull_source:
        RenderVirtualGeometryNodeAndClusterCullSource,
    pub(in crate::graphics::scene::scene_renderer::core) node_and_cluster_cull_record_count: u32,
    pub(in crate::graphics::scene::scene_renderer::core) node_and_cluster_cull_global_state:
        Option<RenderVirtualGeometryNodeAndClusterCullGlobalStateSnapshot>,
    pub(in crate::graphics::scene::scene_renderer::core) node_and_cluster_cull_dispatch_group_count:
        [u32; 3],
    pub(in crate::graphics::scene::scene_renderer::core) node_and_cluster_cull_buffer:
        Option<Arc<wgpu::Buffer>>,
    pub(in crate::graphics::scene::scene_renderer::core) node_and_cluster_cull_dispatch_setup_buffer:
        Option<Arc<wgpu::Buffer>>,
    pub(in crate::graphics::scene::scene_renderer::core) node_and_cluster_cull_launch_worklist_buffer:
        Option<Arc<wgpu::Buffer>>,
    pub(in crate::graphics::scene::scene_renderer::core) node_and_cluster_cull_instance_seed_count:
        u32,
    pub(in crate::graphics::scene::scene_renderer::core) node_and_cluster_cull_instance_seed_buffer:
        Option<Arc<wgpu::Buffer>>,
    pub(in crate::graphics::scene::scene_renderer::core) node_and_cluster_cull_instance_work_item_count:
        u32,
    pub(in crate::graphics::scene::scene_renderer::core) node_and_cluster_cull_instance_work_item_buffer:
        Option<Arc<wgpu::Buffer>>,
    pub(in crate::graphics::scene::scene_renderer::core) node_and_cluster_cull_cluster_work_item_count:
        u32,
    pub(in crate::graphics::scene::scene_renderer::core) node_and_cluster_cull_cluster_work_item_buffer:
        Option<Arc<wgpu::Buffer>>,
    pub(in crate::graphics::scene::scene_renderer::core) node_and_cluster_cull_hierarchy_child_id_count:
        u32,
    pub(in crate::graphics::scene::scene_renderer::core) node_and_cluster_cull_hierarchy_child_id_buffer:
        Option<Arc<wgpu::Buffer>>,
    pub(in crate::graphics::scene::scene_renderer::core) node_and_cluster_cull_child_work_item_count:
        u32,
    pub(in crate::graphics::scene::scene_renderer::core) node_and_cluster_cull_child_work_item_buffer:
        Option<Arc<wgpu::Buffer>>,
    pub(in crate::graphics::scene::scene_renderer::core) node_and_cluster_cull_traversal_record_count:
        u32,
    pub(in crate::graphics::scene::scene_renderer::core) node_and_cluster_cull_traversal_record_buffer:
        Option<Arc<wgpu::Buffer>>,
    pub(in crate::graphics::scene::scene_renderer::core) node_and_cluster_cull_page_request_count:
        u32,
    pub(in crate::graphics::scene::scene_renderer::core) node_and_cluster_cull_page_request_ids:
        Vec<u32>,
    pub(in crate::graphics::scene::scene_renderer::core) node_and_cluster_cull_page_request_buffer:
        Option<Arc<wgpu::Buffer>>,
}

pub(in crate::graphics::scene::scene_renderer::core) struct VirtualGeometryRenderPathOutputUpdate {
    pub(in crate::graphics::scene::scene_renderer::core) selected_cluster_count: u32,
    pub(in crate::graphics::scene::scene_renderer::core) selected_cluster_source:
        RenderVirtualGeometrySelectedClusterSource,
    pub(in crate::graphics::scene::scene_renderer::core) selected_cluster_buffer:
        Option<Arc<wgpu::Buffer>>,
    pub(in crate::graphics::scene::scene_renderer::core) visbuffer64_clear_value: u64,
    pub(in crate::graphics::scene::scene_renderer::core) visbuffer64_source:
        RenderVirtualGeometryVisBuffer64Source,
    pub(in crate::graphics::scene::scene_renderer::core) visbuffer64_entry_count: u32,
    pub(in crate::graphics::scene::scene_renderer::core) visbuffer64_buffer:
        Option<Arc<wgpu::Buffer>>,
    pub(in crate::graphics::scene::scene_renderer::core) hardware_rasterization_source:
        RenderVirtualGeometryHardwareRasterizationSource,
    pub(in crate::graphics::scene::scene_renderer::core) hardware_rasterization_record_count: u32,
    pub(in crate::graphics::scene::scene_renderer::core) hardware_rasterization_buffer:
        Option<Arc<wgpu::Buffer>>,
    pub(in crate::graphics::scene::scene_renderer::core) debug_snapshot:
        Option<RenderVirtualGeometryDebugSnapshot>,
}

pub(in crate::graphics::scene::scene_renderer::core) struct VirtualGeometryIndirectOutputUpdate {
    pub(in crate::graphics::scene::scene_renderer::core) indirect_draw_count: u32,
    pub(in crate::graphics::scene::scene_renderer::core) indirect_buffer_count: u32,
    pub(in crate::graphics::scene::scene_renderer::core) indirect_segment_count: u32,
    pub(in crate::graphics::scene::scene_renderer::core) execution_segment_count: u32,
    pub(in crate::graphics::scene::scene_renderer::core) execution_page_count: u32,
    pub(in crate::graphics::scene::scene_renderer::core) execution_resident_segment_count: u32,
    pub(in crate::graphics::scene::scene_renderer::core) execution_pending_segment_count: u32,
    pub(in crate::graphics::scene::scene_renderer::core) execution_missing_segment_count: u32,
    pub(in crate::graphics::scene::scene_renderer::core) execution_repeated_draw_count: u32,
    pub(in crate::graphics::scene::scene_renderer::core) execution_indirect_offsets: Vec<u64>,
    pub(in crate::graphics::scene::scene_renderer::core) mesh_draw_submission_order:
        Vec<(Option<u32>, u64, u32)>,
    pub(in crate::graphics::scene::scene_renderer::core) mesh_draw_submission_records:
        Vec<(u64, u32, u32, usize)>,
    pub(in crate::graphics::scene::scene_renderer::core) mesh_draw_submission_token_records:
        Vec<(u64, u32, u32, u32, usize)>,
    pub(in crate::graphics::scene::scene_renderer::core) indirect_args_buffer:
        Option<Arc<wgpu::Buffer>>,
    pub(in crate::graphics::scene::scene_renderer::core) indirect_args_count: u32,
    pub(in crate::graphics::scene::scene_renderer::core) indirect_submission_buffer:
        Option<Arc<wgpu::Buffer>>,
    pub(in crate::graphics::scene::scene_renderer::core) indirect_authority_buffer:
        Option<Arc<wgpu::Buffer>>,
    pub(in crate::graphics::scene::scene_renderer::core) indirect_draw_refs_buffer:
        Option<Arc<wgpu::Buffer>>,
    pub(in crate::graphics::scene::scene_renderer::core) indirect_segments_buffer:
        Option<Arc<wgpu::Buffer>>,
    pub(in crate::graphics::scene::scene_renderer::core) indirect_execution_submission_buffer:
        Option<Arc<wgpu::Buffer>>,
    pub(in crate::graphics::scene::scene_renderer::core) indirect_execution_args_buffer:
        Option<Arc<wgpu::Buffer>>,
    pub(in crate::graphics::scene::scene_renderer::core) indirect_execution_authority_buffer:
        Option<Arc<wgpu::Buffer>>,
}

pub(in crate::graphics::scene::scene_renderer::core) struct VirtualGeometryLastOutputUpdate {
    pub(in crate::graphics::scene::scene_renderer::core) node_and_cluster_cull:
        VirtualGeometryCullOutputUpdate,
    pub(in crate::graphics::scene::scene_renderer::core) render_path:
        VirtualGeometryRenderPathOutputUpdate,
    pub(in crate::graphics::scene::scene_renderer::core) indirect:
        VirtualGeometryIndirectOutputUpdate,
}

impl SceneRendererAdvancedPluginOutputs {
    pub(in crate::graphics::scene::scene_renderer::core) fn reset(&mut self) {
        *self = Self::default();
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn take_hybrid_gi_gpu_readback(
        &mut self,
    ) -> Option<HybridGiGpuReadback> {
        self.hybrid_gi_gpu_readback.take()
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn has_virtual_geometry_gpu_readback(
        &self,
    ) -> bool {
        self.virtual_geometry_gpu_readback.is_some()
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn virtual_geometry_gpu_readback_mut(
        &mut self,
    ) -> Option<&mut VirtualGeometryGpuReadback> {
        self.virtual_geometry_gpu_readback.as_mut()
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn virtual_geometry_gpu_readback(
        &self,
    ) -> Option<&VirtualGeometryGpuReadback> {
        self.virtual_geometry_gpu_readback.as_ref()
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn take_virtual_geometry_gpu_readback(
        &mut self,
    ) -> Option<VirtualGeometryGpuReadback> {
        self.virtual_geometry_gpu_readback.take()
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn virtual_geometry_indirect_draw_count(
        &self,
    ) -> u32 {
        self.virtual_geometry_indirect_draw_count
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn virtual_geometry_indirect_buffer_count(
        &self,
    ) -> u32 {
        self.virtual_geometry_indirect_buffer_count
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn virtual_geometry_indirect_segment_count(
        &self,
    ) -> u32 {
        self.virtual_geometry_indirect_segment_count
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn virtual_geometry_indirect_args_count(
        &self,
    ) -> u32 {
        self.virtual_geometry_indirect_args_count
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn virtual_geometry_cluster_selection_input_source(
        &self,
    ) -> RenderVirtualGeometryClusterSelectionInputSource {
        self.virtual_geometry_cluster_selection_input_source
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn virtual_geometry_selected_cluster_source(
        &self,
    ) -> RenderVirtualGeometrySelectedClusterSource {
        self.virtual_geometry_selected_cluster_source
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn virtual_geometry_node_and_cluster_cull_source(
        &self,
    ) -> RenderVirtualGeometryNodeAndClusterCullSource {
        self.virtual_geometry_node_and_cluster_cull_source
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn virtual_geometry_node_and_cluster_cull_record_count(
        &self,
    ) -> u32 {
        self.virtual_geometry_node_and_cluster_cull_record_count
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn virtual_geometry_node_and_cluster_cull_dispatch_group_count(
        &self,
    ) -> [u32; 3] {
        self.virtual_geometry_node_and_cluster_cull_dispatch_group_count
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn virtual_geometry_node_and_cluster_cull_instance_seed_count(
        &self,
    ) -> u32 {
        self.virtual_geometry_node_and_cluster_cull_instance_seed_count
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn virtual_geometry_node_and_cluster_cull_instance_work_item_count(
        &self,
    ) -> u32 {
        self.virtual_geometry_node_and_cluster_cull_instance_work_item_count
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn virtual_geometry_node_and_cluster_cull_cluster_work_item_count(
        &self,
    ) -> u32 {
        self.virtual_geometry_node_and_cluster_cull_cluster_work_item_count
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn virtual_geometry_node_and_cluster_cull_hierarchy_child_id_count(
        &self,
    ) -> u32 {
        self.virtual_geometry_node_and_cluster_cull_hierarchy_child_id_count
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn virtual_geometry_node_and_cluster_cull_child_work_item_count(
        &self,
    ) -> u32 {
        self.virtual_geometry_node_and_cluster_cull_child_work_item_count
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn virtual_geometry_node_and_cluster_cull_traversal_record_count(
        &self,
    ) -> u32 {
        self.virtual_geometry_node_and_cluster_cull_traversal_record_count
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn virtual_geometry_node_and_cluster_cull_page_request_ids(
        &self,
    ) -> &[u32] {
        &self.virtual_geometry_node_and_cluster_cull_page_request_ids
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn virtual_geometry_node_and_cluster_cull_page_request_count(
        &self,
    ) -> u32 {
        self.virtual_geometry_node_and_cluster_cull_page_request_count
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn virtual_geometry_selected_cluster_count(
        &self,
    ) -> u32 {
        self.virtual_geometry_selected_cluster_count
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn virtual_geometry_visbuffer64_source(
        &self,
    ) -> RenderVirtualGeometryVisBuffer64Source {
        self.virtual_geometry_visbuffer64_source
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn virtual_geometry_visbuffer64_entry_count(
        &self,
    ) -> u32 {
        self.virtual_geometry_visbuffer64_entry_count
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn virtual_geometry_hardware_rasterization_source(
        &self,
    ) -> RenderVirtualGeometryHardwareRasterizationSource {
        self.virtual_geometry_hardware_rasterization_source
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn virtual_geometry_hardware_rasterization_record_count(
        &self,
    ) -> u32 {
        self.virtual_geometry_hardware_rasterization_record_count
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn virtual_geometry_execution_segment_count(
        &self,
    ) -> u32 {
        self.virtual_geometry_execution_segment_count
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn virtual_geometry_execution_page_count(
        &self,
    ) -> u32 {
        self.virtual_geometry_execution_page_count
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn virtual_geometry_execution_resident_segment_count(
        &self,
    ) -> u32 {
        self.virtual_geometry_execution_resident_segment_count
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn virtual_geometry_execution_pending_segment_count(
        &self,
    ) -> u32 {
        self.virtual_geometry_execution_pending_segment_count
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn virtual_geometry_execution_missing_segment_count(
        &self,
    ) -> u32 {
        self.virtual_geometry_execution_missing_segment_count
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn virtual_geometry_execution_repeated_draw_count(
        &self,
    ) -> u32 {
        self.virtual_geometry_execution_repeated_draw_count
    }

    #[cfg(test)]
    pub(in crate::graphics::scene::scene_renderer::core) fn virtual_geometry_execution_indirect_offsets(
        &self,
    ) -> Vec<u64> {
        self.virtual_geometry_execution_indirect_offsets.clone()
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn previous_virtual_geometry_node_and_cluster_cull_global_state(
        &self,
    ) -> Option<RenderVirtualGeometryNodeAndClusterCullGlobalStateSnapshot> {
        self.virtual_geometry_debug_snapshot
            .as_ref()
            .and_then(|snapshot| snapshot.node_and_cluster_cull_global_state.clone())
            .or_else(|| {
                self.virtual_geometry_node_and_cluster_cull_global_state
                    .clone()
            })
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn store_virtual_geometry_last_outputs(
        &mut self,
        update: VirtualGeometryLastOutputUpdate,
    ) {
        let VirtualGeometryLastOutputUpdate {
            node_and_cluster_cull,
            render_path,
            indirect,
        } = update;

        self.store_virtual_geometry_cull_outputs(node_and_cluster_cull);
        self.store_virtual_geometry_render_path_outputs(render_path);
        self.store_virtual_geometry_indirect_outputs(indirect);
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn store_virtual_geometry_cull_outputs(
        &mut self,
        update: VirtualGeometryCullOutputUpdate,
    ) {
        self.virtual_geometry_cluster_selection_input_source =
            update.cluster_selection_input_source;
        self.virtual_geometry_cull_input_buffer = update.cull_input_buffer;
        self.virtual_geometry_node_and_cluster_cull_source = update.node_and_cluster_cull_source;
        self.virtual_geometry_node_and_cluster_cull_record_count =
            update.node_and_cluster_cull_record_count;
        self.virtual_geometry_node_and_cluster_cull_global_state =
            update.node_and_cluster_cull_global_state;
        self.virtual_geometry_node_and_cluster_cull_dispatch_group_count =
            update.node_and_cluster_cull_dispatch_group_count;
        self.virtual_geometry_node_and_cluster_cull_buffer = update.node_and_cluster_cull_buffer;
        self.virtual_geometry_node_and_cluster_cull_dispatch_setup_buffer =
            update.node_and_cluster_cull_dispatch_setup_buffer;
        self.virtual_geometry_node_and_cluster_cull_launch_worklist_buffer =
            update.node_and_cluster_cull_launch_worklist_buffer;
        self.virtual_geometry_node_and_cluster_cull_instance_seed_count =
            update.node_and_cluster_cull_instance_seed_count;
        self.virtual_geometry_node_and_cluster_cull_instance_seed_buffer =
            update.node_and_cluster_cull_instance_seed_buffer;
        self.virtual_geometry_node_and_cluster_cull_instance_work_item_count =
            update.node_and_cluster_cull_instance_work_item_count;
        self.virtual_geometry_node_and_cluster_cull_instance_work_item_buffer =
            update.node_and_cluster_cull_instance_work_item_buffer;
        self.virtual_geometry_node_and_cluster_cull_cluster_work_item_count =
            update.node_and_cluster_cull_cluster_work_item_count;
        self.virtual_geometry_node_and_cluster_cull_cluster_work_item_buffer =
            update.node_and_cluster_cull_cluster_work_item_buffer;
        self.virtual_geometry_node_and_cluster_cull_hierarchy_child_id_count =
            update.node_and_cluster_cull_hierarchy_child_id_count;
        self.virtual_geometry_node_and_cluster_cull_hierarchy_child_id_buffer =
            update.node_and_cluster_cull_hierarchy_child_id_buffer;
        self.virtual_geometry_node_and_cluster_cull_child_work_item_count =
            update.node_and_cluster_cull_child_work_item_count;
        self.virtual_geometry_node_and_cluster_cull_child_work_item_buffer =
            update.node_and_cluster_cull_child_work_item_buffer;
        self.virtual_geometry_node_and_cluster_cull_traversal_record_count =
            update.node_and_cluster_cull_traversal_record_count;
        self.virtual_geometry_node_and_cluster_cull_traversal_record_buffer =
            update.node_and_cluster_cull_traversal_record_buffer;
        self.virtual_geometry_node_and_cluster_cull_page_request_count =
            update.node_and_cluster_cull_page_request_count;
        self.virtual_geometry_node_and_cluster_cull_page_request_ids =
            update.node_and_cluster_cull_page_request_ids;
        self.virtual_geometry_node_and_cluster_cull_page_request_buffer =
            update.node_and_cluster_cull_page_request_buffer;
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn store_virtual_geometry_render_path_outputs(
        &mut self,
        update: VirtualGeometryRenderPathOutputUpdate,
    ) {
        self.virtual_geometry_selected_cluster_count = update.selected_cluster_count;
        self.virtual_geometry_selected_cluster_source = update.selected_cluster_source;
        self.virtual_geometry_selected_cluster_buffer = update.selected_cluster_buffer;
        self.virtual_geometry_visbuffer64_clear_value = update.visbuffer64_clear_value;
        self.virtual_geometry_visbuffer64_source = update.visbuffer64_source;
        self.virtual_geometry_visbuffer64_entry_count = update.visbuffer64_entry_count;
        self.virtual_geometry_visbuffer64_buffer = update.visbuffer64_buffer;
        self.virtual_geometry_hardware_rasterization_source = update.hardware_rasterization_source;
        self.virtual_geometry_hardware_rasterization_record_count =
            update.hardware_rasterization_record_count;
        self.virtual_geometry_hardware_rasterization_buffer = update.hardware_rasterization_buffer;
        self.virtual_geometry_debug_snapshot = update.debug_snapshot;
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn store_virtual_geometry_indirect_outputs(
        &mut self,
        update: VirtualGeometryIndirectOutputUpdate,
    ) {
        self.virtual_geometry_indirect_draw_count = update.indirect_draw_count;
        self.virtual_geometry_indirect_buffer_count = update.indirect_buffer_count;
        self.virtual_geometry_indirect_segment_count = update.indirect_segment_count;
        self.virtual_geometry_execution_segment_count = update.execution_segment_count;
        self.virtual_geometry_execution_page_count = update.execution_page_count;
        self.virtual_geometry_execution_resident_segment_count =
            update.execution_resident_segment_count;
        self.virtual_geometry_execution_pending_segment_count =
            update.execution_pending_segment_count;
        self.virtual_geometry_execution_missing_segment_count =
            update.execution_missing_segment_count;
        self.virtual_geometry_execution_repeated_draw_count = update.execution_repeated_draw_count;
        self.virtual_geometry_execution_indirect_offsets = update.execution_indirect_offsets;
        self.virtual_geometry_mesh_draw_submission_order = update.mesh_draw_submission_order;
        self.virtual_geometry_mesh_draw_submission_records = update.mesh_draw_submission_records;
        self.virtual_geometry_mesh_draw_submission_token_records =
            update.mesh_draw_submission_token_records;
        self.virtual_geometry_indirect_args_buffer = update.indirect_args_buffer;
        self.virtual_geometry_indirect_args_count = update.indirect_args_count;
        self.virtual_geometry_indirect_submission_buffer = update.indirect_submission_buffer;
        self.virtual_geometry_indirect_authority_buffer = update.indirect_authority_buffer;
        self.virtual_geometry_indirect_draw_refs_buffer = update.indirect_draw_refs_buffer;
        self.virtual_geometry_indirect_segments_buffer = update.indirect_segments_buffer;
        self.virtual_geometry_indirect_execution_submission_buffer =
            update.indirect_execution_submission_buffer;
        self.virtual_geometry_indirect_execution_args_buffer =
            update.indirect_execution_args_buffer;
        self.virtual_geometry_indirect_execution_authority_buffer =
            update.indirect_execution_authority_buffer;
    }
}

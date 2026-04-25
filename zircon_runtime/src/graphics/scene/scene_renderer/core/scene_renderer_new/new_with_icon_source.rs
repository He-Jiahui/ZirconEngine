use std::collections::HashMap;
use std::sync::Arc;

use crate::asset::pipeline::manager::ProjectAssetManager;
use crate::core::framework::render::{
    RenderVirtualGeometryClusterSelectionInputSource,
    RenderVirtualGeometryHardwareRasterizationSource,
    RenderVirtualGeometryNodeAndClusterCullSource, RenderVirtualGeometrySelectedClusterSource,
    RenderVirtualGeometryVisBuffer64Source,
};

use crate::graphics::types::GraphicsError;

use super::super::super::super::resources::ResourceStreamer;
use super::super::super::graph_execution::{
    RenderGraphExecutionRecord, RenderPassExecutorRegistry,
};
use super::super::super::overlay::ViewportIconSource;
use super::super::constants::OFFSCREEN_FORMAT;
use super::super::scene_renderer::SceneRenderer;
use super::super::scene_renderer_core::SceneRendererCore;

impl SceneRenderer {
    pub(crate) fn new_with_icon_source(
        asset_manager: Arc<ProjectAssetManager>,
        icon_source: Arc<dyn ViewportIconSource>,
    ) -> Result<Self, GraphicsError> {
        let backend = crate::graphics::backend::RenderBackend::new_offscreen()?;
        let core = SceneRendererCore::new_with_icon_source(
            asset_manager.clone(),
            &backend.device,
            &backend.queue,
            OFFSCREEN_FORMAT,
            icon_source,
        );
        let streamer = ResourceStreamer::new(
            asset_manager,
            &backend.device,
            &backend.queue,
            &core.texture_bind_group_layout,
        );

        Ok(Self {
            backend,
            core,
            streamer,
            target: None,
            history_targets: HashMap::new(),
            generation: 0,
            render_pass_executors: RenderPassExecutorRegistry::with_builtin_noop_executors(),
            last_render_graph_execution: RenderGraphExecutionRecord::default(),
            last_hybrid_gi_gpu_readback: None,
            last_virtual_geometry_gpu_readback: None,
            last_virtual_geometry_debug_snapshot: None,
            last_virtual_geometry_indirect_draw_count: 0,
            last_virtual_geometry_indirect_buffer_count: 0,
            last_virtual_geometry_indirect_segment_count: 0,
            last_virtual_geometry_execution_segment_count: 0,
            last_virtual_geometry_execution_page_count: 0,
            last_virtual_geometry_execution_resident_segment_count: 0,
            last_virtual_geometry_execution_pending_segment_count: 0,
            last_virtual_geometry_execution_missing_segment_count: 0,
            last_virtual_geometry_execution_repeated_draw_count: 0,
            last_virtual_geometry_execution_indirect_offsets: Vec::new(),
            last_virtual_geometry_mesh_draw_submission_order: Vec::new(),
            last_virtual_geometry_mesh_draw_submission_records: Vec::new(),
            last_virtual_geometry_mesh_draw_submission_token_records: Vec::new(),
            last_virtual_geometry_indirect_args_buffer: None,
            last_virtual_geometry_indirect_args_count: 0,
            last_virtual_geometry_indirect_submission_buffer: None,
            last_virtual_geometry_indirect_authority_buffer: None,
            last_virtual_geometry_indirect_draw_refs_buffer: None,
            last_virtual_geometry_indirect_segments_buffer: None,
            last_virtual_geometry_indirect_execution_submission_buffer: None,
            last_virtual_geometry_indirect_execution_args_buffer: None,
            last_virtual_geometry_indirect_execution_authority_buffer: None,
            last_virtual_geometry_cluster_selection_input_source:
                RenderVirtualGeometryClusterSelectionInputSource::Unavailable,
            last_virtual_geometry_cull_input_buffer: None,
            last_virtual_geometry_node_and_cluster_cull_source:
                RenderVirtualGeometryNodeAndClusterCullSource::Unavailable,
            last_virtual_geometry_node_and_cluster_cull_record_count: 0,
            last_virtual_geometry_node_and_cluster_cull_global_state: None,
            last_virtual_geometry_node_and_cluster_cull_dispatch_group_count: [0, 0, 0],
            last_virtual_geometry_node_and_cluster_cull_buffer: None,
            last_virtual_geometry_node_and_cluster_cull_dispatch_setup_buffer: None,
            last_virtual_geometry_node_and_cluster_cull_launch_worklist_buffer: None,
            last_virtual_geometry_node_and_cluster_cull_instance_seed_count: 0,
            last_virtual_geometry_node_and_cluster_cull_instance_seed_buffer: None,
            last_virtual_geometry_node_and_cluster_cull_instance_work_item_count: 0,
            last_virtual_geometry_node_and_cluster_cull_instance_work_item_buffer: None,
            last_virtual_geometry_node_and_cluster_cull_cluster_work_item_count: 0,
            last_virtual_geometry_node_and_cluster_cull_cluster_work_item_buffer: None,
            last_virtual_geometry_node_and_cluster_cull_hierarchy_child_id_count: 0,
            last_virtual_geometry_node_and_cluster_cull_hierarchy_child_id_buffer: None,
            last_virtual_geometry_node_and_cluster_cull_child_work_item_count: 0,
            last_virtual_geometry_node_and_cluster_cull_child_work_item_buffer: None,
            last_virtual_geometry_node_and_cluster_cull_traversal_record_count: 0,
            last_virtual_geometry_node_and_cluster_cull_traversal_record_buffer: None,
            last_virtual_geometry_node_and_cluster_cull_page_request_count: 0,
            last_virtual_geometry_node_and_cluster_cull_page_request_ids: Vec::new(),
            last_virtual_geometry_node_and_cluster_cull_page_request_buffer: None,
            last_virtual_geometry_selected_cluster_source:
                RenderVirtualGeometrySelectedClusterSource::Unavailable,
            last_virtual_geometry_selected_cluster_count: 0,
            last_virtual_geometry_selected_cluster_buffer: None,
            last_virtual_geometry_visbuffer64_clear_value: 0,
            last_virtual_geometry_visbuffer64_source:
                RenderVirtualGeometryVisBuffer64Source::Unavailable,
            last_virtual_geometry_visbuffer64_entry_count: 0,
            last_virtual_geometry_visbuffer64_buffer: None,
            last_virtual_geometry_hardware_rasterization_record_count: 0,
            last_virtual_geometry_hardware_rasterization_source:
                RenderVirtualGeometryHardwareRasterizationSource::Unavailable,
            last_virtual_geometry_hardware_rasterization_buffer: None,
        })
    }
}

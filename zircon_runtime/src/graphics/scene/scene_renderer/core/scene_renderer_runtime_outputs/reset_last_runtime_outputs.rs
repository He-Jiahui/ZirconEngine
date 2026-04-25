use super::super::scene_renderer::SceneRenderer;
use crate::core::framework::render::{
    RenderVirtualGeometryClusterSelectionInputSource,
    RenderVirtualGeometryHardwareRasterizationSource,
    RenderVirtualGeometryNodeAndClusterCullSource, RenderVirtualGeometrySelectedClusterSource,
    RenderVirtualGeometryVisBuffer64Source,
};

pub(in crate::graphics::scene::scene_renderer::core) fn reset_last_runtime_outputs(
    renderer: &mut SceneRenderer,
) {
    renderer.last_render_graph_execution = Default::default();
    renderer.last_hybrid_gi_gpu_readback = None;
    renderer.last_virtual_geometry_gpu_readback = None;
    renderer.last_virtual_geometry_debug_snapshot = None;
    renderer.last_virtual_geometry_indirect_draw_count = 0;
    renderer.last_virtual_geometry_indirect_buffer_count = 0;
    renderer.last_virtual_geometry_indirect_segment_count = 0;
    renderer.last_virtual_geometry_execution_segment_count = 0;
    renderer.last_virtual_geometry_execution_page_count = 0;
    renderer.last_virtual_geometry_execution_resident_segment_count = 0;
    renderer.last_virtual_geometry_execution_pending_segment_count = 0;
    renderer.last_virtual_geometry_execution_missing_segment_count = 0;
    renderer.last_virtual_geometry_execution_repeated_draw_count = 0;
    renderer
        .last_virtual_geometry_execution_indirect_offsets
        .clear();
    renderer
        .last_virtual_geometry_mesh_draw_submission_order
        .clear();
    renderer
        .last_virtual_geometry_mesh_draw_submission_records
        .clear();
    renderer
        .last_virtual_geometry_mesh_draw_submission_token_records
        .clear();
    renderer.last_virtual_geometry_indirect_args_buffer = None;
    renderer.last_virtual_geometry_indirect_args_count = 0;
    renderer.last_virtual_geometry_indirect_submission_buffer = None;
    renderer.last_virtual_geometry_indirect_authority_buffer = None;
    renderer.last_virtual_geometry_indirect_draw_refs_buffer = None;
    renderer.last_virtual_geometry_indirect_segments_buffer = None;
    renderer.last_virtual_geometry_indirect_execution_submission_buffer = None;
    renderer.last_virtual_geometry_indirect_execution_args_buffer = None;
    renderer.last_virtual_geometry_indirect_execution_authority_buffer = None;
    renderer.last_virtual_geometry_cluster_selection_input_source =
        RenderVirtualGeometryClusterSelectionInputSource::Unavailable;
    renderer.last_virtual_geometry_cull_input_buffer = None;
    renderer.last_virtual_geometry_node_and_cluster_cull_source =
        RenderVirtualGeometryNodeAndClusterCullSource::Unavailable;
    renderer.last_virtual_geometry_node_and_cluster_cull_record_count = 0;
    renderer.last_virtual_geometry_node_and_cluster_cull_global_state = None;
    renderer.last_virtual_geometry_node_and_cluster_cull_dispatch_group_count = [0, 0, 0];
    renderer.last_virtual_geometry_node_and_cluster_cull_buffer = None;
    renderer.last_virtual_geometry_node_and_cluster_cull_dispatch_setup_buffer = None;
    renderer.last_virtual_geometry_node_and_cluster_cull_launch_worklist_buffer = None;
    renderer.last_virtual_geometry_node_and_cluster_cull_instance_seed_count = 0;
    renderer.last_virtual_geometry_node_and_cluster_cull_instance_seed_buffer = None;
    renderer.last_virtual_geometry_node_and_cluster_cull_instance_work_item_count = 0;
    renderer.last_virtual_geometry_node_and_cluster_cull_instance_work_item_buffer = None;
    renderer.last_virtual_geometry_node_and_cluster_cull_cluster_work_item_count = 0;
    renderer.last_virtual_geometry_node_and_cluster_cull_cluster_work_item_buffer = None;
    renderer.last_virtual_geometry_node_and_cluster_cull_hierarchy_child_id_count = 0;
    renderer.last_virtual_geometry_node_and_cluster_cull_hierarchy_child_id_buffer = None;
    renderer.last_virtual_geometry_node_and_cluster_cull_child_work_item_count = 0;
    renderer.last_virtual_geometry_node_and_cluster_cull_child_work_item_buffer = None;
    renderer.last_virtual_geometry_node_and_cluster_cull_traversal_record_count = 0;
    renderer.last_virtual_geometry_node_and_cluster_cull_traversal_record_buffer = None;
    renderer.last_virtual_geometry_node_and_cluster_cull_page_request_count = 0;
    renderer
        .last_virtual_geometry_node_and_cluster_cull_page_request_ids
        .clear();
    renderer.last_virtual_geometry_node_and_cluster_cull_page_request_buffer = None;
    renderer.last_virtual_geometry_selected_cluster_source =
        RenderVirtualGeometrySelectedClusterSource::Unavailable;
    renderer.last_virtual_geometry_selected_cluster_count = 0;
    renderer.last_virtual_geometry_selected_cluster_buffer = None;
    renderer.last_virtual_geometry_visbuffer64_clear_value = 0;
    renderer.last_virtual_geometry_visbuffer64_source =
        RenderVirtualGeometryVisBuffer64Source::Unavailable;
    renderer.last_virtual_geometry_visbuffer64_entry_count = 0;
    renderer.last_virtual_geometry_visbuffer64_buffer = None;
    renderer.last_virtual_geometry_hardware_rasterization_record_count = 0;
    renderer.last_virtual_geometry_hardware_rasterization_source =
        RenderVirtualGeometryHardwareRasterizationSource::Unavailable;
    renderer.last_virtual_geometry_hardware_rasterization_buffer = None;
}

use crate::core::framework::render::{
    RenderStats, RenderVirtualGeometryClusterSelectionInputSource,
    RenderVirtualGeometryHardwareRasterizationSource,
    RenderVirtualGeometryNodeAndClusterCullSource, RenderVirtualGeometryPayloadSource,
    RenderVirtualGeometrySelectedClusterSource, RenderVirtualGeometryVisBuffer64Source,
};

use super::{DiagnosticStore, record_bool, record_count};

pub(super) fn record(store: &mut DiagnosticStore, stats: &RenderStats) {
    record_input_and_visibility(store, stats);
    record_payload_source(store, stats);
    record_debug(store, stats);
    record_residency(store, stats);
    record_indirect_and_execution(store, stats);
    record_cull_and_outputs(store, stats);
}

fn record_input_and_visibility(store: &mut DiagnosticStore, stats: &RenderStats) {
    let frame_index = stats.submitted_frames;
    record_count(
        store,
        "render.virtual_geometry.cluster_budget",
        frame_index,
        stats.last_virtual_geometry_cluster_budget,
        &["render", "virtual_geometry", "budget"],
    );
    record_count(
        store,
        "render.virtual_geometry.page_budget",
        frame_index,
        stats.last_virtual_geometry_page_budget,
        &["render", "virtual_geometry", "budget"],
    );
    record_count(
        store,
        "render.virtual_geometry.input_cluster_count",
        frame_index,
        stats.last_virtual_geometry_input_cluster_count,
        &["render", "virtual_geometry", "input"],
    );
    record_count(
        store,
        "render.virtual_geometry.input_page_count",
        frame_index,
        stats.last_virtual_geometry_input_page_count,
        &["render", "virtual_geometry", "input"],
    );
    record_count(
        store,
        "render.virtual_geometry.visible_cluster_count",
        frame_index,
        stats.last_virtual_geometry_visible_cluster_count,
        &["render", "virtual_geometry", "visibility"],
    );
    record_count(
        store,
        "render.virtual_geometry.visible_entity_count",
        frame_index,
        stats.last_virtual_geometry_visible_entity_count,
        &["render", "virtual_geometry", "visibility"],
    );
    record_count(
        store,
        "render.virtual_geometry.instance_count",
        frame_index,
        stats.last_virtual_geometry_instance_count,
        &["render", "virtual_geometry", "instance"],
    );
}

fn record_payload_source(store: &mut DiagnosticStore, stats: &RenderStats) {
    let frame_index = stats.submitted_frames;
    let source = stats.last_virtual_geometry_payload_source;
    record_bool(
        store,
        "render.virtual_geometry.payload.source.none",
        frame_index,
        source == RenderVirtualGeometryPayloadSource::None,
        &["render", "virtual_geometry", "payload", "source"],
    );
    record_bool(
        store,
        "render.virtual_geometry.payload.source.authored",
        frame_index,
        source == RenderVirtualGeometryPayloadSource::Authored,
        &["render", "virtual_geometry", "payload", "source"],
    );
    record_bool(
        store,
        "render.virtual_geometry.payload.source.automatic_fallback",
        frame_index,
        source == RenderVirtualGeometryPayloadSource::AutomaticFallback,
        &[
            "render",
            "virtual_geometry",
            "payload",
            "source",
            "fallback",
        ],
    );
}

fn record_debug(store: &mut DiagnosticStore, stats: &RenderStats) {
    let frame_index = stats.submitted_frames;
    record_bool(
        store,
        "render.virtual_geometry.forced_mip_present",
        frame_index,
        stats.last_virtual_geometry_forced_mip.is_some(),
        &["render", "virtual_geometry", "debug", "mip"],
    );
    record_count(
        store,
        "render.virtual_geometry.forced_mip_value",
        frame_index,
        stats
            .last_virtual_geometry_forced_mip
            .map_or(0, usize::from),
        &["render", "virtual_geometry", "debug", "mip"],
    );
    record_bool(
        store,
        "render.virtual_geometry.debug.freeze_cull",
        frame_index,
        stats.last_virtual_geometry_freeze_cull,
        &["render", "virtual_geometry", "debug"],
    );
    record_bool(
        store,
        "render.virtual_geometry.debug.visualize_bvh",
        frame_index,
        stats.last_virtual_geometry_visualize_bvh,
        &["render", "virtual_geometry", "debug"],
    );
    record_bool(
        store,
        "render.virtual_geometry.debug.visualize_visbuffer",
        frame_index,
        stats.last_virtual_geometry_visualize_visbuffer,
        &["render", "virtual_geometry", "debug"],
    );
    record_bool(
        store,
        "render.virtual_geometry.debug.print_leaf_clusters",
        frame_index,
        stats.last_virtual_geometry_print_leaf_clusters,
        &["render", "virtual_geometry", "debug"],
    );
}

fn record_residency(store: &mut DiagnosticStore, stats: &RenderStats) {
    let frame_index = stats.submitted_frames;
    record_count(
        store,
        "render.virtual_geometry.requested_page_count",
        frame_index,
        stats.last_virtual_geometry_requested_page_count,
        &["render", "virtual_geometry", "page", "request"],
    );
    record_count(
        store,
        "render.virtual_geometry.dirty_page_count",
        frame_index,
        stats.last_virtual_geometry_dirty_page_count,
        &["render", "virtual_geometry", "page", "dirty"],
    );
    record_count(
        store,
        "render.virtual_geometry.page_table_entry_count",
        frame_index,
        stats.last_virtual_geometry_page_table_entry_count,
        &["render", "virtual_geometry", "page_table"],
    );
    record_count(
        store,
        "render.virtual_geometry.resident_page_count",
        frame_index,
        stats.last_virtual_geometry_resident_page_count,
        &["render", "virtual_geometry", "page", "resident"],
    );
    record_count(
        store,
        "render.virtual_geometry.pending_request_count",
        frame_index,
        stats.last_virtual_geometry_pending_request_count,
        &["render", "virtual_geometry", "page", "pending"],
    );
    record_count(
        store,
        "render.virtual_geometry.page_dependency_count",
        frame_index,
        stats.last_virtual_geometry_page_dependency_count,
        &["render", "virtual_geometry", "page", "dependency"],
    );
    record_count(
        store,
        "render.virtual_geometry.completed_page_count",
        frame_index,
        stats.last_virtual_geometry_completed_page_count,
        &["render", "virtual_geometry", "page", "completed"],
    );
    record_count(
        store,
        "render.virtual_geometry.replaced_page_count",
        frame_index,
        stats.last_virtual_geometry_replaced_page_count,
        &["render", "virtual_geometry", "page", "replacement"],
    );
}

fn record_indirect_and_execution(store: &mut DiagnosticStore, stats: &RenderStats) {
    let frame_index = stats.submitted_frames;
    record_count(
        store,
        "render.virtual_geometry.indirect_draw_count",
        frame_index,
        stats.last_virtual_geometry_indirect_draw_count,
        &["render", "virtual_geometry", "indirect"],
    );
    record_count(
        store,
        "render.virtual_geometry.indirect_buffer_count",
        frame_index,
        stats.last_virtual_geometry_indirect_buffer_count,
        &["render", "virtual_geometry", "indirect"],
    );
    record_count(
        store,
        "render.virtual_geometry.indirect_args_count",
        frame_index,
        stats.last_virtual_geometry_indirect_args_count,
        &["render", "virtual_geometry", "indirect"],
    );
    record_count(
        store,
        "render.virtual_geometry.indirect_segment_count",
        frame_index,
        stats.last_virtual_geometry_indirect_segment_count,
        &["render", "virtual_geometry", "indirect"],
    );
    record_count(
        store,
        "render.virtual_geometry.execution_segment_count",
        frame_index,
        stats.last_virtual_geometry_execution_segment_count,
        &["render", "virtual_geometry", "execution"],
    );
    record_count(
        store,
        "render.virtual_geometry.execution_page_count",
        frame_index,
        stats.last_virtual_geometry_execution_page_count,
        &["render", "virtual_geometry", "execution", "page"],
    );
    record_count(
        store,
        "render.virtual_geometry.execution_resident_segment_count",
        frame_index,
        stats.last_virtual_geometry_execution_resident_segment_count,
        &["render", "virtual_geometry", "execution", "resident"],
    );
    record_count(
        store,
        "render.virtual_geometry.execution_pending_segment_count",
        frame_index,
        stats.last_virtual_geometry_execution_pending_segment_count,
        &["render", "virtual_geometry", "execution", "pending"],
    );
    record_count(
        store,
        "render.virtual_geometry.execution_missing_segment_count",
        frame_index,
        stats.last_virtual_geometry_execution_missing_segment_count,
        &["render", "virtual_geometry", "execution", "missing"],
    );
    record_count(
        store,
        "render.virtual_geometry.execution_repeated_draw_count",
        frame_index,
        stats.last_virtual_geometry_execution_repeated_draw_count,
        &["render", "virtual_geometry", "execution", "repeat"],
    );
}

fn record_cull_and_outputs(store: &mut DiagnosticStore, stats: &RenderStats) {
    let frame_index = stats.submitted_frames;
    record_cluster_selection_source(store, frame_index, stats);
    record_node_and_cluster_cull(store, stats);
    record_output_sources(store, frame_index, stats);
    record_count(
        store,
        "render.virtual_geometry.selected_cluster_count",
        frame_index,
        stats.last_virtual_geometry_selected_cluster_count,
        &["render", "virtual_geometry", "selected_cluster"],
    );
    record_count(
        store,
        "render.virtual_geometry.visbuffer64_entry_count",
        frame_index,
        stats.last_virtual_geometry_visbuffer64_entry_count,
        &["render", "virtual_geometry", "visbuffer64"],
    );
    record_count(
        store,
        "render.virtual_geometry.hardware_rasterization_record_count",
        frame_index,
        stats.last_virtual_geometry_hardware_rasterization_record_count,
        &["render", "virtual_geometry", "hardware_rasterization"],
    );
}

fn record_cluster_selection_source(
    store: &mut DiagnosticStore,
    frame_index: u64,
    stats: &RenderStats,
) {
    let source = stats.last_virtual_geometry_cluster_selection_input_source;
    record_bool(
        store,
        "render.virtual_geometry.cluster_selection.input_source.unavailable",
        frame_index,
        source == RenderVirtualGeometryClusterSelectionInputSource::Unavailable,
        &["render", "virtual_geometry", "cluster_selection", "source"],
    );
    record_bool(
        store,
        "render.virtual_geometry.cluster_selection.input_source.prepare_on_demand",
        frame_index,
        source == RenderVirtualGeometryClusterSelectionInputSource::PrepareOnDemand,
        &["render", "virtual_geometry", "cluster_selection", "source"],
    );
}

fn record_node_and_cluster_cull(store: &mut DiagnosticStore, stats: &RenderStats) {
    let frame_index = stats.submitted_frames;
    let source = stats.last_virtual_geometry_node_and_cluster_cull_source;
    record_bool(
        store,
        "render.virtual_geometry.node_and_cluster_cull.source.render_path_cull_input",
        frame_index,
        source == RenderVirtualGeometryNodeAndClusterCullSource::RenderPathCullInput,
        &["render", "virtual_geometry", "cull", "source"],
    );
    record_count(
        store,
        "render.virtual_geometry.node_and_cluster_cull.record_count",
        frame_index,
        stats.last_virtual_geometry_node_and_cluster_cull_record_count,
        &["render", "virtual_geometry", "cull"],
    );
    record_count(
        store,
        "render.virtual_geometry.node_and_cluster_cull.dispatch_group_x",
        frame_index,
        stats.last_virtual_geometry_node_and_cluster_cull_dispatch_group_count[0],
        &["render", "virtual_geometry", "cull", "dispatch"],
    );
    record_count(
        store,
        "render.virtual_geometry.node_and_cluster_cull.dispatch_group_y",
        frame_index,
        stats.last_virtual_geometry_node_and_cluster_cull_dispatch_group_count[1],
        &["render", "virtual_geometry", "cull", "dispatch"],
    );
    record_count(
        store,
        "render.virtual_geometry.node_and_cluster_cull.dispatch_group_z",
        frame_index,
        stats.last_virtual_geometry_node_and_cluster_cull_dispatch_group_count[2],
        &["render", "virtual_geometry", "cull", "dispatch"],
    );
    record_count(
        store,
        "render.virtual_geometry.node_and_cluster_cull.instance_seed_count",
        frame_index,
        stats.last_virtual_geometry_node_and_cluster_cull_instance_seed_count,
        &["render", "virtual_geometry", "cull", "instance"],
    );
    record_count(
        store,
        "render.virtual_geometry.node_and_cluster_cull.instance_work_item_count",
        frame_index,
        stats.last_virtual_geometry_node_and_cluster_cull_instance_work_item_count,
        &["render", "virtual_geometry", "cull", "instance"],
    );
    record_count(
        store,
        "render.virtual_geometry.node_and_cluster_cull.cluster_work_item_count",
        frame_index,
        stats.last_virtual_geometry_node_and_cluster_cull_cluster_work_item_count,
        &["render", "virtual_geometry", "cull", "cluster"],
    );
    record_count(
        store,
        "render.virtual_geometry.node_and_cluster_cull.hierarchy_child_id_count",
        frame_index,
        stats.last_virtual_geometry_node_and_cluster_cull_hierarchy_child_id_count,
        &["render", "virtual_geometry", "cull", "hierarchy"],
    );
    record_count(
        store,
        "render.virtual_geometry.node_and_cluster_cull.child_work_item_count",
        frame_index,
        stats.last_virtual_geometry_node_and_cluster_cull_child_work_item_count,
        &["render", "virtual_geometry", "cull", "child"],
    );
    record_count(
        store,
        "render.virtual_geometry.node_and_cluster_cull.traversal_record_count",
        frame_index,
        stats.last_virtual_geometry_node_and_cluster_cull_traversal_record_count,
        &["render", "virtual_geometry", "cull", "traversal"],
    );
    record_count(
        store,
        "render.virtual_geometry.node_and_cluster_cull.page_request_count",
        frame_index,
        stats.last_virtual_geometry_node_and_cluster_cull_page_request_count,
        &["render", "virtual_geometry", "cull", "page"],
    );
}

fn record_output_sources(store: &mut DiagnosticStore, frame_index: u64, stats: &RenderStats) {
    record_bool(
        store,
        "render.virtual_geometry.selected_cluster.source.render_path_execution_selections",
        frame_index,
        stats.last_virtual_geometry_selected_cluster_source
            == RenderVirtualGeometrySelectedClusterSource::RenderPathExecutionSelections,
        &["render", "virtual_geometry", "selected_cluster", "source"],
    );
    record_bool(
        store,
        "render.virtual_geometry.visbuffer64.source.render_path_execution_selections",
        frame_index,
        stats.last_virtual_geometry_visbuffer64_source
            == RenderVirtualGeometryVisBuffer64Source::RenderPathExecutionSelections,
        &["render", "virtual_geometry", "visbuffer64", "source"],
    );
    record_bool(
        store,
        "render.virtual_geometry.hardware_rasterization.source.render_path_execution_selections",
        frame_index,
        stats.last_virtual_geometry_hardware_rasterization_source
            == RenderVirtualGeometryHardwareRasterizationSource::RenderPathExecutionSelections,
        &[
            "render",
            "virtual_geometry",
            "hardware_rasterization",
            "source",
        ],
    );
}

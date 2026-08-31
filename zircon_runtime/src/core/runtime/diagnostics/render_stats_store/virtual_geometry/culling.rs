use crate::core::framework::render::{
    RenderStats, RenderVirtualGeometryClusterSelectionInputSource,
    RenderVirtualGeometryHardwareRasterizationSource,
    RenderVirtualGeometryNodeAndClusterCullSource, RenderVirtualGeometrySelectedClusterSource,
    RenderVirtualGeometryVisBuffer64Source,
};

use super::super::{record_bool, record_count, DiagnosticStore};

pub(super) fn record(store: &mut DiagnosticStore, stats: &RenderStats) {
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

use crate::core::framework::render::{
    RenderHybridGiReadbackOutputs, RenderHybridGiScenePrepareReadbackOutputs,
    RenderPluginRendererOutputs, RenderVirtualGeometryNodeClusterCullReadbackOutputs,
    RenderVirtualGeometryReadbackOutputs,
};

/// 合并 runtime-prepare 与图执行插件结果：列表累积、GI 计数求和、粒子结果保持单所有者。
/// 新增字段时必须明确其跨阶段合并语义，不能默认套用列表追加。
pub(in crate::graphics::scene::scene_renderer::core) fn merge_plugin_renderer_outputs(
    base: &mut RenderPluginRendererOutputs,
    incoming: RenderPluginRendererOutputs,
) {
    if !incoming.virtual_geometry.is_empty() {
        merge_virtual_geometry_outputs(&mut base.virtual_geometry, incoming.virtual_geometry);
    }
    if !incoming.hybrid_gi.is_empty() {
        merge_hybrid_gi_outputs(&mut base.hybrid_gi, incoming.hybrid_gi);
    }
    if !incoming.particles.is_empty() {
        base.particles = incoming.particles;
    }
}

fn merge_virtual_geometry_outputs(
    base: &mut RenderVirtualGeometryReadbackOutputs,
    incoming: RenderVirtualGeometryReadbackOutputs,
) {
    if base.is_empty() {
        *base = incoming;
        return;
    }

    let RenderVirtualGeometryReadbackOutputs {
        mut page_table_entries,
        mut completed_page_assignments,
        mut page_replacements,
        mut selected_clusters,
        mut visbuffer64_entries,
        mut hardware_rasterization_records,
        node_cluster_cull,
    } = incoming;

    base.page_table_entries.append(&mut page_table_entries);
    base.completed_page_assignments
        .append(&mut completed_page_assignments);
    base.page_replacements.append(&mut page_replacements);
    base.selected_clusters.append(&mut selected_clusters);
    base.visbuffer64_entries.append(&mut visbuffer64_entries);
    base.hardware_rasterization_records
        .append(&mut hardware_rasterization_records);
    append_virtual_geometry_node_cluster_cull(&mut base.node_cluster_cull, node_cluster_cull);
}

fn append_virtual_geometry_node_cluster_cull(
    base: &mut RenderVirtualGeometryNodeClusterCullReadbackOutputs,
    incoming: RenderVirtualGeometryNodeClusterCullReadbackOutputs,
) {
    let RenderVirtualGeometryNodeClusterCullReadbackOutputs {
        mut traversal_records,
        mut child_work_items,
        mut cluster_work_items,
        mut launch_worklist_snapshots,
        mut page_request_ids,
    } = incoming;

    base.traversal_records.append(&mut traversal_records);
    base.child_work_items.append(&mut child_work_items);
    base.cluster_work_items.append(&mut cluster_work_items);
    base.launch_worklist_snapshots
        .append(&mut launch_worklist_snapshots);
    base.page_request_ids.append(&mut page_request_ids);
}

fn merge_hybrid_gi_outputs(
    base: &mut RenderHybridGiReadbackOutputs,
    incoming: RenderHybridGiReadbackOutputs,
) {
    if base.is_empty() {
        *base = incoming;
        return;
    }

    let RenderHybridGiReadbackOutputs {
        mut cache_entries,
        mut completed_probe_ids,
        mut completed_trace_region_ids,
        mut probe_irradiance_rgb,
        mut probe_rt_lighting_rgb,
        radiance_cache_gpu_stage_dispatch_counts,
        global_sdf_stats,
        scene_prepare,
    } = incoming;

    base.cache_entries.append(&mut cache_entries);
    base.completed_probe_ids.append(&mut completed_probe_ids);
    base.completed_trace_region_ids
        .append(&mut completed_trace_region_ids);
    base.probe_irradiance_rgb.append(&mut probe_irradiance_rgb);
    base.probe_rt_lighting_rgb
        .append(&mut probe_rt_lighting_rgb);
    for (base_count, incoming_count) in base
        .radiance_cache_gpu_stage_dispatch_counts
        .iter_mut()
        .zip(radiance_cache_gpu_stage_dispatch_counts)
    {
        *base_count = base_count.saturating_add(incoming_count);
    }
    if global_sdf_stats.is_some() {
        base.global_sdf_stats = global_sdf_stats;
    }
    append_hybrid_gi_scene_prepare(&mut base.scene_prepare, scene_prepare);
}

fn append_hybrid_gi_scene_prepare(
    base: &mut RenderHybridGiScenePrepareReadbackOutputs,
    incoming: RenderHybridGiScenePrepareReadbackOutputs,
) {
    let RenderHybridGiScenePrepareReadbackOutputs {
        mut occupied_atlas_slots,
        mut occupied_capture_slots,
        mut atlas_samples,
        mut capture_samples,
        mut surface_cache_depth_samples,
        mut surface_cache_pages,
        mut voxel_clipmaps,
        mut voxel_clipmap_ids,
        mut voxel_samples,
        mut voxel_occupancy,
        mut voxel_occupancy_masks,
        mut voxel_cells,
        mut voxel_cell_samples,
        mut voxel_cell_dominant_nodes,
        mut voxel_cell_dominant_samples,
        mut probe_trace_tiles,
        mut probe_trace_diagnostics,
        probe_trace_dispatch,
        texture_width,
        texture_height,
        texture_layers,
    } = incoming;

    base.occupied_atlas_slots.append(&mut occupied_atlas_slots);
    base.occupied_capture_slots
        .append(&mut occupied_capture_slots);
    base.atlas_samples.append(&mut atlas_samples);
    base.capture_samples.append(&mut capture_samples);
    base.surface_cache_depth_samples
        .append(&mut surface_cache_depth_samples);
    base.surface_cache_pages.append(&mut surface_cache_pages);
    base.voxel_clipmaps.append(&mut voxel_clipmaps);
    base.voxel_clipmap_ids.append(&mut voxel_clipmap_ids);
    base.voxel_samples.append(&mut voxel_samples);
    base.voxel_occupancy.append(&mut voxel_occupancy);
    base.voxel_occupancy_masks
        .append(&mut voxel_occupancy_masks);
    base.voxel_cells.append(&mut voxel_cells);
    base.voxel_cell_samples.append(&mut voxel_cell_samples);
    base.voxel_cell_dominant_nodes
        .append(&mut voxel_cell_dominant_nodes);
    base.voxel_cell_dominant_samples
        .append(&mut voxel_cell_dominant_samples);
    base.probe_trace_tiles.append(&mut probe_trace_tiles);
    base.probe_trace_diagnostics
        .append(&mut probe_trace_diagnostics);
    base.probe_trace_dispatch = [
        base.probe_trace_dispatch[0].max(probe_trace_dispatch[0]),
        base.probe_trace_dispatch[1].max(probe_trace_dispatch[1]),
        base.probe_trace_dispatch[2].max(probe_trace_dispatch[2]),
    ];
    base.texture_width = base.texture_width.max(texture_width);
    base.texture_height = base.texture_height.max(texture_height);
    base.texture_layers = base.texture_layers.max(texture_layers);
}

#[cfg(test)]
#[path = "tests/merge_plugin_renderer_outputs.rs"]
mod tests;

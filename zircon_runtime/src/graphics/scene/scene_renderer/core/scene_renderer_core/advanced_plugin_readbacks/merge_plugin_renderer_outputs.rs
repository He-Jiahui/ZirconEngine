use crate::core::framework::render::{
    RenderHybridGiReadbackOutputs, RenderHybridGiScenePrepareReadbackOutputs,
    RenderPluginRendererOutputs, RenderVirtualGeometryNodeClusterCullReadbackOutputs,
    RenderVirtualGeometryReadbackOutputs,
};

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
        page_table_entries,
        completed_page_assignments,
        page_replacements,
        selected_clusters,
        visbuffer64_entries,
        hardware_rasterization_records,
        node_cluster_cull,
    } = incoming;

    base.page_table_entries.extend(page_table_entries);
    base.completed_page_assignments
        .extend(completed_page_assignments);
    base.page_replacements.extend(page_replacements);
    base.selected_clusters.extend(selected_clusters);
    base.visbuffer64_entries.extend(visbuffer64_entries);
    base.hardware_rasterization_records
        .extend(hardware_rasterization_records);
    append_virtual_geometry_node_cluster_cull(&mut base.node_cluster_cull, node_cluster_cull);
}

fn append_virtual_geometry_node_cluster_cull(
    base: &mut RenderVirtualGeometryNodeClusterCullReadbackOutputs,
    incoming: RenderVirtualGeometryNodeClusterCullReadbackOutputs,
) {
    let RenderVirtualGeometryNodeClusterCullReadbackOutputs {
        traversal_records,
        child_work_items,
        cluster_work_items,
        launch_worklist_snapshots,
        page_request_ids,
    } = incoming;

    base.traversal_records.extend(traversal_records);
    base.child_work_items.extend(child_work_items);
    base.cluster_work_items.extend(cluster_work_items);
    base.launch_worklist_snapshots
        .extend(launch_worklist_snapshots);
    base.page_request_ids.extend(page_request_ids);
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
        cache_entries,
        completed_probe_ids,
        completed_trace_region_ids,
        probe_irradiance_rgb,
        probe_rt_lighting_rgb,
        scene_prepare,
    } = incoming;

    base.cache_entries.extend(cache_entries);
    base.completed_probe_ids.extend(completed_probe_ids);
    base.completed_trace_region_ids
        .extend(completed_trace_region_ids);
    base.probe_irradiance_rgb.extend(probe_irradiance_rgb);
    base.probe_rt_lighting_rgb.extend(probe_rt_lighting_rgb);
    append_hybrid_gi_scene_prepare(&mut base.scene_prepare, scene_prepare);
}

fn append_hybrid_gi_scene_prepare(
    base: &mut RenderHybridGiScenePrepareReadbackOutputs,
    incoming: RenderHybridGiScenePrepareReadbackOutputs,
) {
    let RenderHybridGiScenePrepareReadbackOutputs {
        occupied_atlas_slots,
        occupied_capture_slots,
        atlas_samples,
        capture_samples,
        surface_cache_depth_samples,
        surface_cache_pages,
        voxel_clipmaps,
        voxel_clipmap_ids,
        voxel_samples,
        voxel_occupancy,
        voxel_occupancy_masks,
        voxel_cells,
        voxel_cell_samples,
        voxel_cell_dominant_nodes,
        voxel_cell_dominant_samples,
        probe_trace_tiles,
        probe_trace_dispatch,
        texture_width,
        texture_height,
        texture_layers,
    } = incoming;

    base.occupied_atlas_slots.extend(occupied_atlas_slots);
    base.occupied_capture_slots.extend(occupied_capture_slots);
    base.atlas_samples.extend(atlas_samples);
    base.capture_samples.extend(capture_samples);
    base.surface_cache_depth_samples
        .extend(surface_cache_depth_samples);
    base.surface_cache_pages.extend(surface_cache_pages);
    base.voxel_clipmaps.extend(voxel_clipmaps);
    base.voxel_clipmap_ids.extend(voxel_clipmap_ids);
    base.voxel_samples.extend(voxel_samples);
    base.voxel_occupancy.extend(voxel_occupancy);
    base.voxel_occupancy_masks.extend(voxel_occupancy_masks);
    base.voxel_cells.extend(voxel_cells);
    base.voxel_cell_samples.extend(voxel_cell_samples);
    base.voxel_cell_dominant_nodes
        .extend(voxel_cell_dominant_nodes);
    base.voxel_cell_dominant_samples
        .extend(voxel_cell_dominant_samples);
    base.probe_trace_tiles.extend(probe_trace_tiles);
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
mod tests {
    use super::merge_plugin_renderer_outputs;
    use crate::core::framework::render::{
        RenderHybridGiCacheEntryRecord, RenderHybridGiReadbackOutputs,
        RenderHybridGiScenePrepareReadbackOutputs, RenderHybridGiScenePrepareSample,
        RenderParticleGpuReadbackOutputs, RenderPluginRendererOutputs,
        RenderVirtualGeometryNodeClusterCullReadbackOutputs,
        RenderVirtualGeometryPageAssignmentRecord, RenderVirtualGeometryReadbackOutputs,
    };

    #[test]
    fn merge_preserves_hybrid_gi_runtime_prepare_and_graph_payloads() {
        let mut base = RenderPluginRendererOutputs {
            hybrid_gi: RenderHybridGiReadbackOutputs {
                cache_entries: vec![RenderHybridGiCacheEntryRecord { key: 5, value: 7 }],
                scene_prepare: RenderHybridGiScenePrepareReadbackOutputs {
                    surface_cache_depth_samples: vec![RenderHybridGiScenePrepareSample {
                        index: 1,
                        rgba8: [96, 96, 96, 255],
                    }],
                    probe_trace_dispatch: [1, 1, 1],
                    texture_width: 64,
                    ..RenderHybridGiScenePrepareReadbackOutputs::default()
                },
                ..RenderHybridGiReadbackOutputs::default()
            },
            ..RenderPluginRendererOutputs::default()
        };

        merge_plugin_renderer_outputs(
            &mut base,
            RenderPluginRendererOutputs {
                hybrid_gi: RenderHybridGiReadbackOutputs {
                    completed_probe_ids: vec![11],
                    scene_prepare: RenderHybridGiScenePrepareReadbackOutputs {
                        atlas_samples: vec![RenderHybridGiScenePrepareSample {
                            index: 2,
                            rgba8: [10, 20, 30, 255],
                        }],
                        probe_trace_dispatch: [1, 1, 3],
                        texture_width: 128,
                        ..RenderHybridGiScenePrepareReadbackOutputs::default()
                    },
                    ..RenderHybridGiReadbackOutputs::default()
                },
                ..RenderPluginRendererOutputs::default()
            },
        );

        assert_eq!(
            base.hybrid_gi.cache_entries,
            vec![RenderHybridGiCacheEntryRecord { key: 5, value: 7 }]
        );
        assert_eq!(base.hybrid_gi.completed_probe_ids, vec![11]);
        assert_eq!(base.hybrid_gi.scene_prepare.atlas_samples.len(), 1);
        assert_eq!(
            base.hybrid_gi
                .scene_prepare
                .surface_cache_depth_samples
                .len(),
            1
        );
        assert_eq!(base.hybrid_gi.scene_prepare.probe_trace_dispatch, [1, 1, 3]);
        assert_eq!(base.hybrid_gi.scene_prepare.texture_width, 128);
    }

    #[test]
    fn merge_appends_virtual_geometry_readback_payloads() {
        let mut base = RenderPluginRendererOutputs {
            virtual_geometry: RenderVirtualGeometryReadbackOutputs {
                completed_page_assignments: vec![RenderVirtualGeometryPageAssignmentRecord {
                    page_id: 4,
                    physical_slot: 1,
                }],
                ..RenderVirtualGeometryReadbackOutputs::default()
            },
            ..RenderPluginRendererOutputs::default()
        };

        merge_plugin_renderer_outputs(
            &mut base,
            RenderPluginRendererOutputs {
                virtual_geometry: RenderVirtualGeometryReadbackOutputs {
                    page_table_entries: vec![9],
                    node_cluster_cull: RenderVirtualGeometryNodeClusterCullReadbackOutputs {
                        page_request_ids: vec![300],
                        ..RenderVirtualGeometryNodeClusterCullReadbackOutputs::default()
                    },
                    ..RenderVirtualGeometryReadbackOutputs::default()
                },
                ..RenderPluginRendererOutputs::default()
            },
        );

        assert_eq!(base.virtual_geometry.completed_page_assignments.len(), 1);
        assert_eq!(base.virtual_geometry.page_table_entries, vec![9]);
        assert_eq!(
            base.virtual_geometry.node_cluster_cull.page_request_ids,
            vec![300]
        );
    }

    #[test]
    fn merge_keeps_particle_outputs_single_owner() {
        let mut base = RenderPluginRendererOutputs {
            particles: RenderParticleGpuReadbackOutputs {
                alive_count: 2,
                spawned_total: 2,
                ..RenderParticleGpuReadbackOutputs::default()
            },
            ..RenderPluginRendererOutputs::default()
        };

        merge_plugin_renderer_outputs(
            &mut base,
            RenderPluginRendererOutputs {
                particles: RenderParticleGpuReadbackOutputs {
                    alive_count: 5,
                    spawned_total: 5,
                    ..RenderParticleGpuReadbackOutputs::default()
                },
                ..RenderPluginRendererOutputs::default()
            },
        );

        assert_eq!(base.particles.alive_count, 5);
        assert_eq!(base.particles.spawned_total, 5);
    }
}

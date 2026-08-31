use super::{
    RenderHybridGiProbeTraceDiagnosticRecord, RenderHybridGiReadbackOutputs,
    RenderHybridGiScenePrepareReadbackOutputs, RenderHybridGiScenePrepareSample,
    RenderHybridGiSurfaceCachePageRecord, RenderHybridGiTraceTileRecord,
    RenderHybridGiVoxelClipmapRecord, RenderParticleGpuReadbackOutputs,
    RenderPluginRendererOutputs, RenderVirtualGeometryNodeClusterCullReadbackOutputs,
    RenderVirtualGeometryReadbackOutputs,
};
use crate::core::framework::render::{
    RenderVirtualGeometryExecutionState,
    RenderVirtualGeometryNodeAndClusterCullTraversalChildSource,
    RenderVirtualGeometryNodeAndClusterCullTraversalOp,
    RenderVirtualGeometryNodeAndClusterCullTraversalRecord, RenderVirtualGeometryVisBuffer64Entry,
};

#[test]
fn default_plugin_renderer_outputs_are_empty() {
    let outputs = RenderPluginRendererOutputs::default();

    assert!(outputs.virtual_geometry.page_table_entries.is_empty());
    assert!(outputs.virtual_geometry.selected_clusters.is_empty());
    assert!(outputs.hybrid_gi.completed_probe_ids.is_empty());
    assert!(outputs.hybrid_gi.scene_prepare.voxel_cells.is_empty());
    assert_eq!(outputs.particles.alive_count, 0);
    assert!(outputs.particles.per_emitter_spawned.is_empty());
    assert!(outputs.is_empty());
    assert!(outputs.virtual_geometry.is_empty());
    assert!(outputs.hybrid_gi.is_empty());
}

#[test]
fn particle_gpu_readback_outputs_are_empty_only_without_payloads() {
    let empty = RenderParticleGpuReadbackOutputs::default();
    assert!(empty.is_empty());

    let with_alive_count = RenderParticleGpuReadbackOutputs {
        alive_count: 4,
        ..RenderParticleGpuReadbackOutputs::default()
    };
    assert!(!with_alive_count.is_empty());

    let with_indirect_args = RenderParticleGpuReadbackOutputs {
        indirect_draw_args: [6, 4, 0, 0],
        ..RenderParticleGpuReadbackOutputs::default()
    };
    assert!(!with_indirect_args.is_empty());
}

#[test]
fn virtual_geometry_readback_outputs_report_node_cluster_cull_payloads() {
    let outputs = RenderVirtualGeometryReadbackOutputs {
        node_cluster_cull: RenderVirtualGeometryNodeClusterCullReadbackOutputs {
            traversal_records: vec![RenderVirtualGeometryNodeAndClusterCullTraversalRecord {
                op: RenderVirtualGeometryNodeAndClusterCullTraversalOp::VisitNode,
                child_source: RenderVirtualGeometryNodeAndClusterCullTraversalChildSource::None,
                instance_index: 0,
                entity: 0,
                cluster_array_index: 0,
                hierarchy_node_id: None,
                node_cluster_start: 0,
                node_cluster_count: 0,
                child_base: 0,
                child_count: 0,
                traversal_index: 0,
                cluster_budget: 0,
                page_budget: 0,
                forced_mip: None,
            }],
            page_request_ids: vec![300, 301],
            ..RenderVirtualGeometryNodeClusterCullReadbackOutputs::default()
        },
        ..RenderVirtualGeometryReadbackOutputs::default()
    };

    assert!(!outputs.node_cluster_cull.is_empty());
    assert!(!outputs.is_empty());

    let mut outputs = outputs;
    assert_eq!(
        outputs.take_node_and_cluster_cull_page_request_ids(),
        vec![300, 301]
    );
    assert!(outputs.node_cluster_cull.page_request_ids.is_empty());

    let outputs = RenderVirtualGeometryReadbackOutputs {
        visbuffer64_entries: vec![RenderVirtualGeometryVisBuffer64Entry {
            entry_index: 0,
            packed_value: RenderVirtualGeometryVisBuffer64Entry::CLEAR_VALUE,
            instance_index: None,
            entity: 0,
            cluster_id: 0,
            page_id: 0,
            lod_level: 0,
            state: RenderVirtualGeometryExecutionState::Missing,
        }],
        ..RenderVirtualGeometryReadbackOutputs::default()
    };

    assert!(!outputs.is_empty());
}

#[test]
fn hybrid_gi_readback_outputs_ignore_non_runtime_scene_prepare_metadata_for_feedback() {
    let outputs = RenderHybridGiReadbackOutputs {
        scene_prepare: RenderHybridGiScenePrepareReadbackOutputs {
            occupied_atlas_slots: vec![1],
            texture_width: 64,
            texture_height: 64,
            texture_layers: 2,
            ..RenderHybridGiScenePrepareReadbackOutputs::default()
        },
        ..RenderHybridGiReadbackOutputs::default()
    };

    assert!(!outputs.scene_prepare.has_runtime_feedback_payload());
    assert!(outputs.is_empty());
}

#[test]
fn hybrid_gi_global_sdf_stats_are_runtime_feedback_even_when_zeroed() {
    let outputs = RenderHybridGiReadbackOutputs {
        global_sdf_stats: Some(Default::default()),
        ..RenderHybridGiReadbackOutputs::default()
    };

    assert!(!outputs.is_empty());
}

#[test]
fn hybrid_gi_readback_outputs_report_scene_prepare_runtime_payloads() {
    let outputs = RenderHybridGiReadbackOutputs {
        scene_prepare: RenderHybridGiScenePrepareReadbackOutputs {
            voxel_samples: vec![RenderHybridGiScenePrepareSample {
                index: 4,
                rgba8: [8, 16, 24, 255],
            }],
            ..RenderHybridGiScenePrepareReadbackOutputs::default()
        },
        ..RenderHybridGiReadbackOutputs::default()
    };

    assert!(outputs.scene_prepare.has_runtime_feedback_payload());
    assert!(!outputs.is_empty());
}

#[test]
fn hybrid_gi_readback_outputs_report_world_space_trace_lookup_records() {
    let outputs = RenderHybridGiReadbackOutputs {
        scene_prepare: RenderHybridGiScenePrepareReadbackOutputs {
            surface_cache_pages: vec![RenderHybridGiSurfaceCachePageRecord {
                page_id: 3,
                owner_card_id: 7,
                atlas_slot_id: 11,
                bounds_center_x_bits: 1.0_f32.to_bits(),
                bounds_center_y_bits: 2.0_f32.to_bits(),
                bounds_center_z_bits: 3.0_f32.to_bits(),
                bounds_radius_bits: 4.0_f32.to_bits(),
                radiance_rgba8: [24, 48, 96, 255],
            }],
            voxel_clipmaps: vec![RenderHybridGiVoxelClipmapRecord {
                clipmap_id: 5,
                center_x_bits: 0.0_f32.to_bits(),
                center_y_bits: 1.0_f32.to_bits(),
                center_z_bits: 2.0_f32.to_bits(),
                half_extent_bits: 8.0_f32.to_bits(),
            }],
            ..RenderHybridGiScenePrepareReadbackOutputs::default()
        },
        ..RenderHybridGiReadbackOutputs::default()
    };

    assert!(outputs.scene_prepare.has_runtime_feedback_payload());
    assert!(!outputs.is_empty());
}

#[test]
fn hybrid_gi_readback_outputs_report_depth_and_trace_tile_payloads() {
    let outputs = RenderHybridGiReadbackOutputs {
        scene_prepare: RenderHybridGiScenePrepareReadbackOutputs {
            surface_cache_depth_samples: vec![RenderHybridGiScenePrepareSample {
                index: 2,
                rgba8: [96, 96, 96, 255],
            }],
            probe_trace_tiles: vec![RenderHybridGiTraceTileRecord {
                tile_id: 0,
                probe_id: 4,
                trace_region_id: 9,
                ray_count: 32,
            }],
            probe_trace_dispatch: [1, 1, 1],
            ..RenderHybridGiScenePrepareReadbackOutputs::default()
        },
        ..RenderHybridGiReadbackOutputs::default()
    };

    assert!(outputs.scene_prepare.has_runtime_feedback_payload());
    assert!(!outputs.is_empty());
}

#[test]
fn hybrid_gi_readback_outputs_report_probe_trace_diagnostics() {
    let outputs = RenderHybridGiReadbackOutputs {
        scene_prepare: RenderHybridGiScenePrepareReadbackOutputs {
            probe_trace_diagnostics: vec![RenderHybridGiProbeTraceDiagnosticRecord {
                probe_id: 7,
                ..RenderHybridGiProbeTraceDiagnosticRecord::default()
            }],
            ..RenderHybridGiScenePrepareReadbackOutputs::default()
        },
        ..RenderHybridGiReadbackOutputs::default()
    };

    assert!(outputs.scene_prepare.has_runtime_feedback_payload());
    assert!(!outputs.is_empty());
}

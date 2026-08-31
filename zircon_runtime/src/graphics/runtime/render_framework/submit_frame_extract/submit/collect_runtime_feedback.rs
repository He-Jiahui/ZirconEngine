use crate::core::framework::render::{
    RenderHybridGiReadbackOutputs, RenderParticleGpuReadbackOutputs,
    RenderPreparedRuntimeSidebands, RenderVirtualGeometryReadbackOutputs,
};
use crate::graphics::{
    HybridGiGpuCompletion, HybridGiRuntimeFeedback, ParticleGpuFeedback, ParticleRuntimeFeedback,
    SceneRenderer, VirtualGeometryGpuCompletion, VirtualGeometryRuntimeFeedback,
};

use super::super::frame_submission_context::FrameSubmissionContext;
use super::super::runtime_feedback_batch::RuntimeFeedbackBatch;

pub(super) fn collect_runtime_feedback(
    renderer: &mut SceneRenderer,
    context: &FrameSubmissionContext,
    sidebands: &mut RenderPreparedRuntimeSidebands,
) -> RuntimeFeedbackBatch {
    RuntimeFeedbackBatch::new(
        collect_hybrid_gi_feedback(renderer, context, sidebands),
        collect_particle_feedback(renderer, sidebands),
        collect_virtual_geometry_feedback(renderer, context, sidebands),
    )
}

fn collect_hybrid_gi_feedback(
    renderer: &mut SceneRenderer,
    context: &FrameSubmissionContext,
    sidebands: &mut RenderPreparedRuntimeSidebands,
) -> HybridGiRuntimeFeedback {
    let readback_outputs = merge_hybrid_gi_readback_outputs(
        renderer.take_last_hybrid_gi_readback_outputs(),
        sidebands.take_hybrid_gi_readback_outputs(),
    );

    HybridGiRuntimeFeedback::new(
        HybridGiGpuCompletion::from_readback_outputs(readback_outputs),
        context.hybrid_gi_feedback().cloned(),
    )
    .with_evictable_probe_ids(sidebands.take_hybrid_gi_evictable_probe_ids())
}

fn collect_particle_feedback(
    renderer: &mut SceneRenderer,
    sidebands: &mut RenderPreparedRuntimeSidebands,
) -> ParticleRuntimeFeedback {
    let readback_outputs = merge_particle_readback_outputs(
        renderer.take_last_particle_gpu_readback_outputs(),
        sidebands.take_particle_readback_outputs(),
    );
    let gpu_feedback =
        (!readback_outputs.is_empty()).then(|| ParticleGpuFeedback::new(readback_outputs));

    ParticleRuntimeFeedback::new(gpu_feedback)
}

fn collect_virtual_geometry_feedback(
    renderer: &mut SceneRenderer,
    context: &FrameSubmissionContext,
    sidebands: &mut RenderPreparedRuntimeSidebands,
) -> VirtualGeometryRuntimeFeedback {
    let mut readback_outputs = merge_virtual_geometry_readback_outputs(
        renderer.take_last_virtual_geometry_readback_outputs(),
        sidebands.take_virtual_geometry_readback_outputs(),
    );
    let node_and_cluster_cull_page_requests =
        readback_outputs.take_node_and_cluster_cull_page_request_ids();

    VirtualGeometryRuntimeFeedback::new(
        VirtualGeometryGpuCompletion::from_readback_outputs(readback_outputs),
        node_and_cluster_cull_page_requests,
        context.virtual_geometry_feedback().cloned(),
        context.predicted_generation(),
    )
    .with_evictable_page_ids(sidebands.take_virtual_geometry_evictable_page_ids())
}

fn merge_hybrid_gi_readback_outputs(
    mut renderer_outputs: RenderHybridGiReadbackOutputs,
    sideband_outputs: RenderHybridGiReadbackOutputs,
) -> RenderHybridGiReadbackOutputs {
    if renderer_outputs.is_empty() {
        return sideband_outputs;
    }
    if sideband_outputs.is_empty() {
        return renderer_outputs;
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
    } = sideband_outputs;
    renderer_outputs.cache_entries.append(&mut cache_entries);
    renderer_outputs
        .completed_probe_ids
        .append(&mut completed_probe_ids);
    renderer_outputs
        .completed_trace_region_ids
        .append(&mut completed_trace_region_ids);
    renderer_outputs
        .probe_irradiance_rgb
        .append(&mut probe_irradiance_rgb);
    renderer_outputs
        .probe_rt_lighting_rgb
        .append(&mut probe_rt_lighting_rgb);
    for (renderer_count, sideband_count) in renderer_outputs
        .radiance_cache_gpu_stage_dispatch_counts
        .iter_mut()
        .zip(radiance_cache_gpu_stage_dispatch_counts)
    {
        *renderer_count = renderer_count.saturating_add(sideband_count);
    }
    if renderer_outputs.global_sdf_stats.is_none() {
        renderer_outputs.global_sdf_stats = global_sdf_stats;
    }
    if renderer_outputs
        .scene_prepare
        .has_runtime_feedback_payload()
    {
        append_hybrid_gi_scene_prepare_readback(&mut renderer_outputs.scene_prepare, scene_prepare);
    } else {
        renderer_outputs.scene_prepare = scene_prepare;
    }
    renderer_outputs
}

fn append_hybrid_gi_scene_prepare_readback(
    renderer_outputs: &mut crate::core::framework::render::RenderHybridGiScenePrepareReadbackOutputs,
    sideband_outputs: crate::core::framework::render::RenderHybridGiScenePrepareReadbackOutputs,
) {
    let crate::core::framework::render::RenderHybridGiScenePrepareReadbackOutputs {
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
    } = sideband_outputs;

    renderer_outputs
        .occupied_atlas_slots
        .append(&mut occupied_atlas_slots);
    renderer_outputs
        .occupied_capture_slots
        .append(&mut occupied_capture_slots);
    renderer_outputs.atlas_samples.append(&mut atlas_samples);
    renderer_outputs
        .capture_samples
        .append(&mut capture_samples);
    renderer_outputs
        .surface_cache_depth_samples
        .append(&mut surface_cache_depth_samples);
    renderer_outputs
        .surface_cache_pages
        .append(&mut surface_cache_pages);
    renderer_outputs.voxel_clipmaps.append(&mut voxel_clipmaps);
    renderer_outputs
        .voxel_clipmap_ids
        .append(&mut voxel_clipmap_ids);
    renderer_outputs.voxel_samples.append(&mut voxel_samples);
    renderer_outputs
        .voxel_occupancy
        .append(&mut voxel_occupancy);
    renderer_outputs
        .voxel_occupancy_masks
        .append(&mut voxel_occupancy_masks);
    renderer_outputs.voxel_cells.append(&mut voxel_cells);
    renderer_outputs
        .voxel_cell_samples
        .append(&mut voxel_cell_samples);
    renderer_outputs
        .voxel_cell_dominant_nodes
        .append(&mut voxel_cell_dominant_nodes);
    renderer_outputs
        .voxel_cell_dominant_samples
        .append(&mut voxel_cell_dominant_samples);
    renderer_outputs
        .probe_trace_tiles
        .append(&mut probe_trace_tiles);
    renderer_outputs
        .probe_trace_diagnostics
        .append(&mut probe_trace_diagnostics);
    renderer_outputs.probe_trace_dispatch = [
        renderer_outputs.probe_trace_dispatch[0].max(probe_trace_dispatch[0]),
        renderer_outputs.probe_trace_dispatch[1].max(probe_trace_dispatch[1]),
        renderer_outputs.probe_trace_dispatch[2].max(probe_trace_dispatch[2]),
    ];
    renderer_outputs.texture_width = renderer_outputs.texture_width.max(texture_width);
    renderer_outputs.texture_height = renderer_outputs.texture_height.max(texture_height);
    renderer_outputs.texture_layers = renderer_outputs.texture_layers.max(texture_layers);
}

fn merge_particle_readback_outputs(
    renderer_outputs: RenderParticleGpuReadbackOutputs,
    sideband_outputs: RenderParticleGpuReadbackOutputs,
) -> RenderParticleGpuReadbackOutputs {
    if !renderer_outputs.is_empty() {
        return renderer_outputs;
    }

    sideband_outputs
}

fn merge_virtual_geometry_readback_outputs(
    mut renderer_outputs: RenderVirtualGeometryReadbackOutputs,
    sideband_outputs: RenderVirtualGeometryReadbackOutputs,
) -> RenderVirtualGeometryReadbackOutputs {
    if renderer_outputs.is_empty() {
        return sideband_outputs;
    }
    if sideband_outputs.is_empty() {
        return renderer_outputs;
    }

    let RenderVirtualGeometryReadbackOutputs {
        page_table_entries,
        completed_page_assignments,
        page_replacements,
        selected_clusters,
        visbuffer64_entries,
        hardware_rasterization_records,
        node_cluster_cull,
    } = sideband_outputs;
    let crate::core::framework::render::RenderVirtualGeometryNodeClusterCullReadbackOutputs {
        traversal_records,
        child_work_items,
        cluster_work_items,
        launch_worklist_snapshots,
        page_request_ids,
    } = node_cluster_cull;

    renderer_outputs
        .page_table_entries
        .extend(page_table_entries);
    renderer_outputs
        .completed_page_assignments
        .extend(completed_page_assignments);
    renderer_outputs.page_replacements.extend(page_replacements);
    renderer_outputs.selected_clusters.extend(selected_clusters);
    renderer_outputs
        .visbuffer64_entries
        .extend(visbuffer64_entries);
    renderer_outputs
        .hardware_rasterization_records
        .extend(hardware_rasterization_records);
    renderer_outputs
        .node_cluster_cull
        .traversal_records
        .extend(traversal_records);
    renderer_outputs
        .node_cluster_cull
        .child_work_items
        .extend(child_work_items);
    renderer_outputs
        .node_cluster_cull
        .cluster_work_items
        .extend(cluster_work_items);
    renderer_outputs
        .node_cluster_cull
        .launch_worklist_snapshots
        .extend(launch_worklist_snapshots);
    renderer_outputs
        .node_cluster_cull
        .page_request_ids
        .extend(page_request_ids);
    renderer_outputs
}

#[cfg(test)]
mod tests {
    use std::hint::black_box;
    use std::time::Instant;

    use super::{
        merge_hybrid_gi_readback_outputs, merge_particle_readback_outputs,
        merge_virtual_geometry_readback_outputs,
    };
    use crate::core::framework::render::{
        RenderHybridGiGlobalSdfStats, RenderHybridGiReadbackOutputs,
        RenderHybridGiScenePrepareReadbackOutputs, RenderHybridGiScenePrepareSample,
        RenderParticleGpuReadbackOutputs, RenderVirtualGeometryNodeClusterCullReadbackOutputs,
        RenderVirtualGeometryPageAssignmentRecord, RenderVirtualGeometryReadbackOutputs,
    };

    #[test]
    fn merge_hybrid_gi_sideband_preserves_renderer_and_prepare_payloads() {
        let merged = merge_hybrid_gi_readback_outputs(
            RenderHybridGiReadbackOutputs {
                completed_probe_ids: vec![10],
                radiance_cache_gpu_stage_dispatch_counts: [1, 2, 3, 4, 5, 6],
                scene_prepare: RenderHybridGiScenePrepareReadbackOutputs {
                    atlas_samples: vec![RenderHybridGiScenePrepareSample {
                        index: 1,
                        rgba8: [1, 2, 3, 255],
                    }],
                    texture_width: 32,
                    ..RenderHybridGiScenePrepareReadbackOutputs::default()
                },
                ..RenderHybridGiReadbackOutputs::default()
            },
            RenderHybridGiReadbackOutputs {
                completed_probe_ids: vec![11],
                radiance_cache_gpu_stage_dispatch_counts: [6, 5, 4, 3, 2, 1],
                scene_prepare: RenderHybridGiScenePrepareReadbackOutputs {
                    voxel_samples: vec![RenderHybridGiScenePrepareSample {
                        index: 4,
                        rgba8: [4, 5, 6, 255],
                    }],
                    texture_width: 64,
                    ..RenderHybridGiScenePrepareReadbackOutputs::default()
                },
                ..RenderHybridGiReadbackOutputs::default()
            },
        );

        assert_eq!(merged.completed_probe_ids, vec![10, 11]);
        assert_eq!(merged.radiance_cache_gpu_stage_dispatch_counts, [7; 6]);
        assert_eq!(merged.scene_prepare.atlas_samples.len(), 1);
        assert_eq!(merged.scene_prepare.voxel_samples.len(), 1);
        assert_eq!(merged.scene_prepare.texture_width, 64);
    }

    #[test]
    fn merge_hybrid_gi_sideband_preserves_renderer_global_sdf_stats() {
        let merged = merge_hybrid_gi_readback_outputs(
            RenderHybridGiReadbackOutputs {
                global_sdf_stats: Some(RenderHybridGiGlobalSdfStats {
                    resident_page_count: 8,
                    uploaded_page_count: 2,
                    ..RenderHybridGiGlobalSdfStats::default()
                }),
                ..RenderHybridGiReadbackOutputs::default()
            },
            RenderHybridGiReadbackOutputs {
                global_sdf_stats: Some(RenderHybridGiGlobalSdfStats {
                    resident_page_count: 1,
                    ..RenderHybridGiGlobalSdfStats::default()
                }),
                ..RenderHybridGiReadbackOutputs::default()
            },
        );

        let stats = merged
            .global_sdf_stats
            .expect("renderer Global SDF stats must remain authoritative");
        assert_eq!(stats.resident_page_count, 8);
        assert_eq!(stats.uploaded_page_count, 2);
    }

    #[test]
    fn merge_virtual_geometry_sideband_preserves_node_cluster_page_requests() {
        let merged = merge_virtual_geometry_readback_outputs(
            RenderVirtualGeometryReadbackOutputs {
                completed_page_assignments: vec![RenderVirtualGeometryPageAssignmentRecord {
                    page_id: 42,
                    physical_slot: 3,
                }],
                ..RenderVirtualGeometryReadbackOutputs::default()
            },
            RenderVirtualGeometryReadbackOutputs {
                node_cluster_cull: RenderVirtualGeometryNodeClusterCullReadbackOutputs {
                    page_request_ids: vec![300, 301],
                    ..RenderVirtualGeometryNodeClusterCullReadbackOutputs::default()
                },
                ..RenderVirtualGeometryReadbackOutputs::default()
            },
        );

        assert_eq!(merged.completed_page_assignments.len(), 1);
        assert_eq!(merged.node_cluster_cull.page_request_ids, vec![300, 301]);
    }

    #[test]
    fn merge_particle_sideband_uses_renderer_payload_as_authority() {
        let sideband = RenderParticleGpuReadbackOutputs {
            alive_count: 2,
            spawned_total: 2,
            per_emitter_spawned: vec![2],
            indirect_draw_args: [6, 2, 0, 0],
            ..RenderParticleGpuReadbackOutputs::default()
        };
        let renderer = RenderParticleGpuReadbackOutputs {
            alive_count: 4,
            spawned_total: 4,
            per_emitter_spawned: vec![4],
            indirect_draw_args: [6, 4, 0, 0],
            ..RenderParticleGpuReadbackOutputs::default()
        };

        assert_eq!(
            merge_particle_readback_outputs(
                RenderParticleGpuReadbackOutputs::default(),
                sideband.clone()
            ),
            sideband
        );
        assert_eq!(
            merge_particle_readback_outputs(renderer.clone(), sideband),
            renderer
        );
    }

    #[test]
    fn optimization_batch_dn_hybrid_readback_append_preserves_owned_vectors() {
        let source = include_str!("collect_runtime_feedback.rs");
        let merge = source
            .split("fn merge_hybrid_gi_readback_outputs")
            .nth(1)
            .expect("hybrid readback merge")
            .split("fn merge_particle_readback_outputs")
            .next()
            .expect("hybrid readback merge body");
        let prepare = source
            .split("fn append_hybrid_gi_scene_prepare_readback")
            .nth(1)
            .expect("scene prepare merge")
            .split("fn merge_virtual_geometry_readback_outputs")
            .next()
            .expect("scene prepare merge body");

        assert!(merge.matches(".append(&mut").count() >= 5);
        assert!(prepare.matches(".append(&mut").count() >= 15);
        assert!(!merge.contains(".extend(") && !prepare.contains(".extend("));
    }

    #[test]
    #[ignore = "release-only alternating p95 performance gate"]
    fn optimization_batch_dn_hybrid_readback_append_p95() {
        const SAMPLE_PAIRS: usize = 17;
        const MERGES_PER_SAMPLE: usize = 8_192;
        const VALUES_PER_VECTOR: usize = 4_096;

        let template = (0..VALUES_PER_VECTOR as u64).collect::<Vec<_>>();
        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
        for sample_index in 0..SAMPLE_PAIRS {
            if sample_index % 2 == 0 {
                legacy_samples.push(measure_hybrid_readback_merge(
                    &template,
                    MERGES_PER_SAMPLE,
                    true,
                ));
                optimized_samples.push(measure_hybrid_readback_merge(
                    &template,
                    MERGES_PER_SAMPLE,
                    false,
                ));
            } else {
                optimized_samples.push(measure_hybrid_readback_merge(
                    &template,
                    MERGES_PER_SAMPLE,
                    false,
                ));
                legacy_samples.push(measure_hybrid_readback_merge(
                    &template,
                    MERGES_PER_SAMPLE,
                    true,
                ));
            }
        }

        let legacy_p95 = p95(&mut legacy_samples);
        let optimized_p95 = p95(&mut optimized_samples);
        println!(
            "RUNTIME422_HYBRID_READBACK_APPEND_BENCH_V1 legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} ratio={:.4}",
            optimized_p95 as f64 / legacy_p95.max(1) as f64
        );
        assert!(
            optimized_p95.saturating_mul(100) <= legacy_p95.saturating_mul(70),
            "hybrid readback append p95 {optimized_p95}ns exceeded 70% of legacy {legacy_p95}ns"
        );
    }

    fn measure_hybrid_readback_merge(template: &[u64], merge_count: usize, legacy: bool) -> u128 {
        let started_at = Instant::now();
        let mut checksum = 0_u64;
        for _ in 0..merge_count {
            let mut target = vec![black_box(1_u64)];
            let mut incoming = black_box(template).to_vec();
            if legacy {
                target.extend(incoming);
            } else {
                target.append(&mut incoming);
            }
            checksum = checksum.wrapping_add(black_box(target.len()) as u64);
        }
        black_box(checksum);
        started_at.elapsed().as_nanos()
    }

    fn p95(samples: &mut [u128]) -> u128 {
        samples.sort_unstable();
        samples[(samples.len() * 95).div_ceil(100).saturating_sub(1)]
    }
}

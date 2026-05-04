use crate::core::framework::render::RenderParticleGpuReadbackOutputs;
#[cfg(test)]
use crate::core::framework::render::{
    RenderHybridGiReadbackOutputs, RenderVirtualGeometryReadbackOutputs,
};
use crate::{
    HybridGiGpuCompletion, HybridGiRuntimeFeedback, ParticleGpuFeedback, ParticleRuntimeFeedback,
    SceneRenderer, VirtualGeometryGpuCompletion, VirtualGeometryRuntimeFeedback,
};

use super::super::frame_submission_context::FrameSubmissionContext;
use super::super::prepared_runtime_submission::PreparedRuntimeSubmission;
use super::super::runtime_feedback_batch::RuntimeFeedbackBatch;

pub(super) fn collect_runtime_feedback(
    renderer: &mut SceneRenderer,
    context: &FrameSubmissionContext,
    prepared: &PreparedRuntimeSubmission,
) -> RuntimeFeedbackBatch {
    RuntimeFeedbackBatch::new(
        collect_hybrid_gi_feedback(renderer, context, prepared),
        collect_particle_feedback(renderer, prepared),
        collect_virtual_geometry_feedback(renderer, context, prepared),
    )
}

fn collect_hybrid_gi_feedback(
    renderer: &mut SceneRenderer,
    context: &FrameSubmissionContext,
    _prepared: &PreparedRuntimeSubmission,
) -> HybridGiRuntimeFeedback {
    let readback_outputs = renderer.take_last_hybrid_gi_readback_outputs();

    HybridGiRuntimeFeedback::new(
        HybridGiGpuCompletion::from_readback_outputs(readback_outputs),
        context.hybrid_gi_feedback().cloned(),
    )
}

fn collect_particle_feedback(
    renderer: &mut SceneRenderer,
    prepared: &PreparedRuntimeSubmission,
) -> ParticleRuntimeFeedback {
    let readback_outputs = merge_particle_readback_outputs(
        renderer.take_last_particle_gpu_readback_outputs(),
        prepared.particle_readback_outputs(),
    );
    let gpu_feedback =
        (!readback_outputs.is_empty()).then(|| ParticleGpuFeedback::new(readback_outputs));

    ParticleRuntimeFeedback::new(gpu_feedback)
}

fn collect_virtual_geometry_feedback(
    renderer: &mut SceneRenderer,
    context: &FrameSubmissionContext,
    _prepared: &PreparedRuntimeSubmission,
) -> VirtualGeometryRuntimeFeedback {
    let mut readback_outputs = renderer.take_last_virtual_geometry_readback_outputs();
    let node_and_cluster_cull_page_requests =
        readback_outputs.take_node_and_cluster_cull_page_request_ids();

    VirtualGeometryRuntimeFeedback::new(
        VirtualGeometryGpuCompletion::from_readback_outputs(readback_outputs),
        node_and_cluster_cull_page_requests,
        context.virtual_geometry_feedback().cloned(),
        context.predicted_generation(),
    )
}

#[cfg(test)]
fn merge_hybrid_gi_readback_outputs(
    mut renderer_outputs: RenderHybridGiReadbackOutputs,
    sideband_outputs: &RenderHybridGiReadbackOutputs,
) -> RenderHybridGiReadbackOutputs {
    if renderer_outputs.is_empty() {
        return sideband_outputs.clone();
    }
    if sideband_outputs.is_empty() {
        return renderer_outputs;
    }

    renderer_outputs
        .cache_entries
        .extend(sideband_outputs.cache_entries.iter().cloned());
    renderer_outputs
        .completed_probe_ids
        .extend(sideband_outputs.completed_probe_ids.iter().copied());
    renderer_outputs
        .completed_trace_region_ids
        .extend(sideband_outputs.completed_trace_region_ids.iter().copied());
    renderer_outputs
        .probe_irradiance_rgb
        .extend(sideband_outputs.probe_irradiance_rgb.iter().copied());
    renderer_outputs
        .probe_rt_lighting_rgb
        .extend(sideband_outputs.probe_rt_lighting_rgb.iter().copied());
    if renderer_outputs
        .scene_prepare
        .has_runtime_feedback_payload()
    {
        append_hybrid_gi_scene_prepare_readback(
            &mut renderer_outputs.scene_prepare,
            &sideband_outputs.scene_prepare,
        );
    } else {
        renderer_outputs.scene_prepare = sideband_outputs.scene_prepare.clone();
    }
    renderer_outputs
}

#[cfg(test)]
fn append_hybrid_gi_scene_prepare_readback(
    renderer_outputs: &mut crate::core::framework::render::RenderHybridGiScenePrepareReadbackOutputs,
    sideband_outputs: &crate::core::framework::render::RenderHybridGiScenePrepareReadbackOutputs,
) {
    renderer_outputs
        .occupied_atlas_slots
        .extend(sideband_outputs.occupied_atlas_slots.iter().copied());
    renderer_outputs
        .occupied_capture_slots
        .extend(sideband_outputs.occupied_capture_slots.iter().copied());
    renderer_outputs
        .atlas_samples
        .extend(sideband_outputs.atlas_samples.iter().cloned());
    renderer_outputs
        .capture_samples
        .extend(sideband_outputs.capture_samples.iter().cloned());
    renderer_outputs
        .voxel_clipmap_ids
        .extend(sideband_outputs.voxel_clipmap_ids.iter().copied());
    renderer_outputs
        .voxel_samples
        .extend(sideband_outputs.voxel_samples.iter().cloned());
    renderer_outputs
        .voxel_occupancy
        .extend(sideband_outputs.voxel_occupancy.iter().copied());
    renderer_outputs
        .voxel_occupancy_masks
        .extend(sideband_outputs.voxel_occupancy_masks.iter().cloned());
    renderer_outputs
        .voxel_cells
        .extend(sideband_outputs.voxel_cells.iter().cloned());
    renderer_outputs
        .voxel_cell_samples
        .extend(sideband_outputs.voxel_cell_samples.iter().cloned());
    renderer_outputs
        .voxel_cell_dominant_nodes
        .extend(sideband_outputs.voxel_cell_dominant_nodes.iter().cloned());
    renderer_outputs
        .voxel_cell_dominant_samples
        .extend(sideband_outputs.voxel_cell_dominant_samples.iter().cloned());
    renderer_outputs.texture_width = renderer_outputs
        .texture_width
        .max(sideband_outputs.texture_width);
    renderer_outputs.texture_height = renderer_outputs
        .texture_height
        .max(sideband_outputs.texture_height);
    renderer_outputs.texture_layers = renderer_outputs
        .texture_layers
        .max(sideband_outputs.texture_layers);
}

fn merge_particle_readback_outputs(
    renderer_outputs: RenderParticleGpuReadbackOutputs,
    sideband_outputs: &RenderParticleGpuReadbackOutputs,
) -> RenderParticleGpuReadbackOutputs {
    if !renderer_outputs.is_empty() {
        return renderer_outputs;
    }

    sideband_outputs.clone()
}

#[cfg(test)]
fn merge_virtual_geometry_readback_outputs(
    mut renderer_outputs: RenderVirtualGeometryReadbackOutputs,
    sideband_outputs: &RenderVirtualGeometryReadbackOutputs,
) -> RenderVirtualGeometryReadbackOutputs {
    if renderer_outputs.is_empty() {
        return sideband_outputs.clone();
    }
    if sideband_outputs.is_empty() {
        return renderer_outputs;
    }

    renderer_outputs
        .page_table_entries
        .extend(sideband_outputs.page_table_entries.iter().copied());
    renderer_outputs
        .completed_page_assignments
        .extend(sideband_outputs.completed_page_assignments.iter().cloned());
    renderer_outputs
        .page_replacements
        .extend(sideband_outputs.page_replacements.iter().cloned());
    renderer_outputs
        .selected_clusters
        .extend(sideband_outputs.selected_clusters.iter().cloned());
    renderer_outputs
        .visbuffer64_entries
        .extend(sideband_outputs.visbuffer64_entries.iter().cloned());
    renderer_outputs.hardware_rasterization_records.extend(
        sideband_outputs
            .hardware_rasterization_records
            .iter()
            .cloned(),
    );
    renderer_outputs.node_cluster_cull.traversal_records.extend(
        sideband_outputs
            .node_cluster_cull
            .traversal_records
            .iter()
            .cloned(),
    );
    renderer_outputs.node_cluster_cull.child_work_items.extend(
        sideband_outputs
            .node_cluster_cull
            .child_work_items
            .iter()
            .cloned(),
    );
    renderer_outputs
        .node_cluster_cull
        .cluster_work_items
        .extend(
            sideband_outputs
                .node_cluster_cull
                .cluster_work_items
                .iter()
                .cloned(),
        );
    renderer_outputs
        .node_cluster_cull
        .launch_worklist_snapshots
        .extend(
            sideband_outputs
                .node_cluster_cull
                .launch_worklist_snapshots
                .iter()
                .cloned(),
        );
    renderer_outputs.node_cluster_cull.page_request_ids.extend(
        sideband_outputs
            .node_cluster_cull
            .page_request_ids
            .iter()
            .copied(),
    );
    renderer_outputs
}

#[cfg(test)]
mod tests {
    use super::{
        merge_hybrid_gi_readback_outputs, merge_particle_readback_outputs,
        merge_virtual_geometry_readback_outputs,
    };
    use crate::core::framework::render::{
        RenderHybridGiReadbackOutputs, RenderHybridGiScenePrepareReadbackOutputs,
        RenderHybridGiScenePrepareSample, RenderParticleGpuReadbackOutputs,
        RenderVirtualGeometryNodeClusterCullReadbackOutputs,
        RenderVirtualGeometryPageAssignmentRecord, RenderVirtualGeometryReadbackOutputs,
    };

    #[test]
    fn merge_hybrid_gi_sideband_preserves_renderer_and_prepare_payloads() {
        let merged = merge_hybrid_gi_readback_outputs(
            RenderHybridGiReadbackOutputs {
                completed_probe_ids: vec![10],
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
            &RenderHybridGiReadbackOutputs {
                completed_probe_ids: vec![11],
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
        assert_eq!(merged.scene_prepare.atlas_samples.len(), 1);
        assert_eq!(merged.scene_prepare.voxel_samples.len(), 1);
        assert_eq!(merged.scene_prepare.texture_width, 64);
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
            &RenderVirtualGeometryReadbackOutputs {
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
            merge_particle_readback_outputs(RenderParticleGpuReadbackOutputs::default(), &sideband),
            sideband
        );
        assert_eq!(
            merge_particle_readback_outputs(renderer.clone(), &sideband),
            renderer
        );
    }
}

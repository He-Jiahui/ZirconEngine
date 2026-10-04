//! 场景执行的 GPU 与可见性反馈汇总后写回相机记录，供下一次 provider 准备消费。
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
#[path = "tests/collect_runtime_feedback.rs"]
mod tests;

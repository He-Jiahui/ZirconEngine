use std::collections::HashMap;
use std::hash::Hash;

use super::mesh_draw::{
    MeshDraw, MeshDrawBatchKey, MeshDrawGeometrySource, MeshDrawQueuePhase, MeshDrawQueueProfile,
};

mod stats;
mod stats_bridge;

#[cfg(test)]
mod stats_bridge_tests;

pub(crate) use self::stats::{
    PreparedMeshQueueStats, PreparedMeshVirtualGeometryExecutionStats,
    PreparedMeshVirtualGeometryIndirectStats,
};

pub(crate) struct PreparedMeshQueue {
    stats: PreparedMeshQueueStats,
}

pub(crate) fn prepare_mesh_queue(draws: &[MeshDraw]) -> PreparedMeshQueue {
    let stats = summarize_prepared_mesh_queue_items::<MeshDrawBatchKey>(draws.iter().map(|draw| {
        (
            draw.queue_profile(),
            draw.casts_shadow(),
            draw.has_previous_velocity_transform(),
            draw.is_skinned(),
            draw.has_skinned_joint_palette_upload(),
            draw.has_previous_skinned_joint_palette_upload(),
            draw.has_skinned_gpu_source_candidate(),
            draw.has_skinned_gpu_cpu_morphed_source_candidate(),
            draw.uses_skinned_gpu_skinning(),
            draw.batch_key(),
        )
    }));

    PreparedMeshQueue { stats }
}

impl PreparedMeshQueue {
    pub(crate) fn stats(&self) -> PreparedMeshQueueStats {
        self.stats
    }
}

pub(crate) fn summarize_prepared_mesh_queue_items<K>(
    items: impl IntoIterator<
        Item = (
            MeshDrawQueueProfile,
            bool,
            bool,
            bool,
            bool,
            bool,
            bool,
            bool,
            bool,
            K,
        ),
    >,
) -> PreparedMeshQueueStats
where
    K: Eq + Hash,
{
    let mut stats = PreparedMeshQueueStats::default();
    let mut candidate_groups = HashMap::<K, CandidateGroupCounts>::new();

    for (
        profile,
        casts_shadow,
        has_previous_velocity_transform,
        is_skinned,
        has_skinned_joint_palette_upload,
        has_previous_skinned_joint_palette_upload,
        has_skinned_gpu_source_candidate,
        has_skinned_gpu_cpu_morphed_source_candidate,
        uses_skinned_gpu_skinning,
        key,
    ) in items
    {
        stats.draw_count += 1;
        let phase = profile.phase();
        match phase {
            MeshDrawQueuePhase::Opaque => stats.opaque_draw_count += 1,
            MeshDrawQueuePhase::AlphaMask => stats.alpha_mask_draw_count += 1,
            MeshDrawQueuePhase::Transparent => stats.transparent_draw_count += 1,
        }
        if casts_shadow {
            stats.shadow_caster_draw_count += 1;
            if matches!(phase, MeshDrawQueuePhase::AlphaMask) {
                stats.alpha_mask_shadow_caster_draw_count += 1;
            }
        }
        if profile.early_z_eligible() {
            stats.early_z_draw_count += 1;
        }
        let geometry_source = profile.geometry_source();
        match geometry_source {
            MeshDrawGeometrySource::Prepared => stats.prepared_geometry_draw_count += 1,
            MeshDrawGeometrySource::Dynamic
            | MeshDrawGeometrySource::DynamicCpuMorphedSource
            | MeshDrawGeometrySource::DynamicGpuMorphedSource
            | MeshDrawGeometrySource::DynamicGpuSkinningSource
            | MeshDrawGeometrySource::DynamicGpuSkinnedMorphedSource
            | MeshDrawGeometrySource::DynamicCpuMorphedGpuSkinningSource => {
                stats.dynamic_geometry_draw_count += 1
            }
        }
        match geometry_source {
            MeshDrawGeometrySource::DynamicGpuMorphedSource => {
                stats.gpu_morphed_source_draw_count += 1;
            }
            MeshDrawGeometrySource::DynamicGpuSkinnedMorphedSource => {
                stats.gpu_skinned_morphed_source_draw_count += 1;
            }
            _ => {}
        }
        if profile.uses_indirect_draw() {
            stats.indirect_draw_count += 1;
        }
        if profile.uses_mesh_lod() {
            stats.lod_draw_count += 1;
        }
        if is_skinned {
            stats.skinned_draw_count += 1;
        }
        if has_skinned_joint_palette_upload {
            stats.skinned_palette_upload_count += 1;
        }
        if has_previous_skinned_joint_palette_upload {
            stats.skinned_previous_palette_upload_count += 1;
        }
        if has_skinned_gpu_source_candidate {
            stats.skinned_gpu_source_candidate_count += 1;
        }
        if has_skinned_gpu_cpu_morphed_source_candidate {
            stats.skinned_gpu_cpu_morphed_source_candidate_count += 1;
        }
        let missing_cpu_morphed_previous_shape_velocity = profile.velocity_history_eligible()
            && uses_skinned_gpu_skinning
            && has_skinned_gpu_cpu_morphed_source_candidate
            && !has_previous_velocity_transform;
        if uses_skinned_gpu_skinning {
            stats.skinned_gpu_skinning_draw_count += 1;
            if has_previous_velocity_transform {
                stats.skinned_gpu_velocity_draw_count += 1;
            }
        }
        if profile.velocity_history_eligible() {
            if has_previous_velocity_transform {
                stats.previous_velocity_transform_draw_count += 1;
            } else if missing_cpu_morphed_previous_shape_velocity {
                stats.skinned_gpu_cpu_morphed_previous_shape_velocity_missing_count += 1;
            } else {
                stats.missing_velocity_transform_draw_count += 1;
            }
        }
        let static_batch_eligible = profile.static_batch_eligible();
        let dynamic_batch_eligible = profile.dynamic_batch_eligible();
        let gpu_instancing_eligible = profile.gpu_instancing_eligible();
        if static_batch_eligible || dynamic_batch_eligible || gpu_instancing_eligible {
            let group = candidate_groups.entry(key).or_default();
            group.static_batch += static_batch_eligible as usize;
            group.dynamic_batch += dynamic_batch_eligible as usize;
            group.gpu_instancing += gpu_instancing_eligible as usize;
        }
    }

    let (groups, draws) =
        repeated_group_stats(candidate_groups.values().map(|group| group.static_batch));
    stats.static_batch_candidate_group_count = groups;
    stats.static_batch_candidate_draw_count = draws;
    let (groups, draws) =
        repeated_group_stats(candidate_groups.values().map(|group| group.dynamic_batch));
    stats.dynamic_batch_candidate_group_count = groups;
    stats.dynamic_batch_candidate_draw_count = draws;
    let (groups, draws) =
        repeated_group_stats(candidate_groups.values().map(|group| group.gpu_instancing));
    stats.gpu_instancing_candidate_group_count = groups;
    stats.gpu_instancing_candidate_draw_count = draws;

    stats
}

#[derive(Default)]
struct CandidateGroupCounts {
    static_batch: usize,
    dynamic_batch: usize,
    gpu_instancing: usize,
}

fn repeated_group_stats(group_sizes: impl IntoIterator<Item = usize>) -> (usize, usize) {
    group_sizes
        .into_iter()
        .filter(|size| *size > 1)
        .fold((0, 0), |(groups, draws), size| (groups + 1, draws + size))
}

#[cfg(test)]
mod tests;

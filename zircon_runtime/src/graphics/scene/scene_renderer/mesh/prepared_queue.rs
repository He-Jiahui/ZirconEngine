use std::collections::HashMap;
use std::hash::Hash;

use super::mesh_draw::{
    MeshDraw, MeshDrawBatchKey, MeshDrawGeometrySource, MeshDrawQueuePhase, MeshDrawQueueProfile,
};

pub(crate) struct PreparedMeshQueue<'a> {
    early_z_draws: Vec<&'a MeshDraw>,
    stats: PreparedMeshQueueStats,
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct PreparedMeshQueueStats {
    pub(crate) draw_count: usize,
    pub(crate) opaque_draw_count: usize,
    pub(crate) alpha_mask_draw_count: usize,
    pub(crate) transparent_draw_count: usize,
    pub(crate) early_z_draw_count: usize,
    pub(crate) prepared_geometry_draw_count: usize,
    pub(crate) dynamic_geometry_draw_count: usize,
    pub(crate) indirect_draw_count: usize,
    pub(crate) static_batch_candidate_group_count: usize,
    pub(crate) static_batch_candidate_draw_count: usize,
    pub(crate) dynamic_batch_candidate_group_count: usize,
    pub(crate) dynamic_batch_candidate_draw_count: usize,
    pub(crate) gpu_instancing_candidate_group_count: usize,
    pub(crate) gpu_instancing_candidate_draw_count: usize,
}

pub(crate) fn prepare_mesh_queue(draws: &[MeshDraw]) -> PreparedMeshQueue<'_> {
    let early_z_draws = draws
        .iter()
        .filter(|draw| draw.queue_profile().early_z_eligible())
        .collect::<Vec<_>>();
    let stats = summarize_prepared_mesh_queue_items::<MeshDrawBatchKey>(
        draws
            .iter()
            .map(|draw| (draw.queue_profile(), draw.batch_key())),
    );

    PreparedMeshQueue {
        early_z_draws,
        stats,
    }
}

impl<'a> PreparedMeshQueue<'a> {
    pub(crate) fn early_z_draws(&self) -> &[&'a MeshDraw] {
        &self.early_z_draws
    }

    pub(crate) fn stats(&self) -> PreparedMeshQueueStats {
        self.stats
    }
}

pub(crate) fn summarize_prepared_mesh_queue_items<K>(
    items: impl IntoIterator<Item = (MeshDrawQueueProfile, K)>,
) -> PreparedMeshQueueStats
where
    K: Clone + Eq + Hash,
{
    let mut stats = PreparedMeshQueueStats::default();
    let mut static_batch_groups = HashMap::<K, usize>::new();
    let mut dynamic_batch_groups = HashMap::<K, usize>::new();
    let mut gpu_instancing_groups = HashMap::<K, usize>::new();

    for (profile, key) in items {
        stats.draw_count += 1;
        match profile.phase() {
            MeshDrawQueuePhase::Opaque => stats.opaque_draw_count += 1,
            MeshDrawQueuePhase::AlphaMask => stats.alpha_mask_draw_count += 1,
            MeshDrawQueuePhase::Transparent => stats.transparent_draw_count += 1,
        }
        if profile.early_z_eligible() {
            stats.early_z_draw_count += 1;
        }
        match profile.geometry_source() {
            MeshDrawGeometrySource::Prepared => stats.prepared_geometry_draw_count += 1,
            MeshDrawGeometrySource::Dynamic => stats.dynamic_geometry_draw_count += 1,
        }
        if profile.uses_indirect_draw() {
            stats.indirect_draw_count += 1;
        }
        if profile.static_batch_eligible() {
            *static_batch_groups.entry(key.clone()).or_default() += 1;
        }
        if profile.dynamic_batch_eligible() {
            *dynamic_batch_groups.entry(key.clone()).or_default() += 1;
        }
        if profile.gpu_instancing_eligible() {
            *gpu_instancing_groups.entry(key).or_default() += 1;
        }
    }

    let (groups, draws) = repeated_group_stats(static_batch_groups.values().copied());
    stats.static_batch_candidate_group_count = groups;
    stats.static_batch_candidate_draw_count = draws;
    let (groups, draws) = repeated_group_stats(dynamic_batch_groups.values().copied());
    stats.dynamic_batch_candidate_group_count = groups;
    stats.dynamic_batch_candidate_draw_count = draws;
    let (groups, draws) = repeated_group_stats(gpu_instancing_groups.values().copied());
    stats.gpu_instancing_candidate_group_count = groups;
    stats.gpu_instancing_candidate_draw_count = draws;

    stats
}

fn repeated_group_stats(group_sizes: impl IntoIterator<Item = usize>) -> (usize, usize) {
    group_sizes
        .into_iter()
        .filter(|size| *size > 1)
        .fold((0, 0), |(groups, draws), size| (groups + 1, draws + size))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::framework::scene::Mobility;

    #[test]
    fn prepared_queue_stats_allow_early_z_only_for_opaque_and_alpha_mask() {
        let stats = summarize_prepared_mesh_queue_items([
            (
                profile(
                    MeshDrawQueuePhase::Opaque,
                    MeshDrawGeometrySource::Prepared,
                    Mobility::Static,
                    false,
                ),
                1_u8,
            ),
            (
                profile(
                    MeshDrawQueuePhase::AlphaMask,
                    MeshDrawGeometrySource::Prepared,
                    Mobility::Static,
                    false,
                ),
                2,
            ),
            (
                profile(
                    MeshDrawQueuePhase::Transparent,
                    MeshDrawGeometrySource::Prepared,
                    Mobility::Static,
                    false,
                ),
                3,
            ),
        ]);

        assert_eq!(stats.draw_count, 3);
        assert_eq!(stats.opaque_draw_count, 1);
        assert_eq!(stats.alpha_mask_draw_count, 1);
        assert_eq!(stats.transparent_draw_count, 1);
        assert_eq!(stats.early_z_draw_count, 2);
    }

    #[test]
    fn prepared_queue_stats_require_repeated_direct_prepared_keys_for_batching() {
        let stats = summarize_prepared_mesh_queue_items([
            (
                profile(
                    MeshDrawQueuePhase::Opaque,
                    MeshDrawGeometrySource::Prepared,
                    Mobility::Static,
                    false,
                ),
                "static-a",
            ),
            (
                profile(
                    MeshDrawQueuePhase::Opaque,
                    MeshDrawGeometrySource::Prepared,
                    Mobility::Static,
                    false,
                ),
                "static-a",
            ),
            (
                profile(
                    MeshDrawQueuePhase::Opaque,
                    MeshDrawGeometrySource::Prepared,
                    Mobility::Dynamic,
                    false,
                ),
                "dynamic-a",
            ),
            (
                profile(
                    MeshDrawQueuePhase::Opaque,
                    MeshDrawGeometrySource::Prepared,
                    Mobility::Dynamic,
                    false,
                ),
                "dynamic-a",
            ),
            (
                profile(
                    MeshDrawQueuePhase::Opaque,
                    MeshDrawGeometrySource::Dynamic,
                    Mobility::Dynamic,
                    false,
                ),
                "dynamic-a",
            ),
            (
                profile(
                    MeshDrawQueuePhase::Opaque,
                    MeshDrawGeometrySource::Prepared,
                    Mobility::Static,
                    true,
                ),
                "static-a",
            ),
            (
                profile(
                    MeshDrawQueuePhase::Transparent,
                    MeshDrawGeometrySource::Prepared,
                    Mobility::Static,
                    false,
                ),
                "static-a",
            ),
        ]);

        assert_eq!(stats.prepared_geometry_draw_count, 6);
        assert_eq!(stats.dynamic_geometry_draw_count, 1);
        assert_eq!(stats.indirect_draw_count, 1);
        assert_eq!(stats.static_batch_candidate_group_count, 1);
        assert_eq!(stats.static_batch_candidate_draw_count, 2);
        assert_eq!(stats.dynamic_batch_candidate_group_count, 1);
        assert_eq!(stats.dynamic_batch_candidate_draw_count, 2);
        assert_eq!(stats.gpu_instancing_candidate_group_count, 2);
        assert_eq!(stats.gpu_instancing_candidate_draw_count, 4);
    }

    fn profile(
        phase: MeshDrawQueuePhase,
        geometry_source: MeshDrawGeometrySource,
        mobility: Mobility,
        uses_indirect_draw: bool,
    ) -> MeshDrawQueueProfile {
        MeshDrawQueueProfile::new(phase, geometry_source, mobility, uses_indirect_draw)
    }
}

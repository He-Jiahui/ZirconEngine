use std::collections::HashMap;
use std::hash::Hash;

use super::mesh_draw::{
    MeshDraw, MeshDrawBatchKey, MeshDrawGeometrySource, MeshDrawQueuePhase, MeshDrawQueueProfile,
};
use super::mesh_pass::{MeshDrawReplayStats, MeshPassCommandBufferStats};
use crate::core::framework::render::RenderGpuSceneUploadPath;
use crate::graphics::scene::gpu_scene::{GpuSceneStats, GpuSceneUploadPath, GpuSceneUploadReport};

pub(crate) struct PreparedMeshQueue {
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
    pub(crate) shadow_caster_draw_count: usize,
    pub(crate) alpha_mask_shadow_caster_draw_count: usize,
    pub(crate) prepared_geometry_draw_count: usize,
    pub(crate) dynamic_geometry_draw_count: usize,
    pub(crate) skinned_draw_count: usize,
    pub(crate) skinned_palette_upload_count: usize,
    pub(crate) skinned_previous_palette_upload_count: usize,
    pub(crate) skinned_gpu_source_candidate_count: usize,
    pub(crate) skinned_gpu_cpu_morphed_source_candidate_count: usize,
    pub(crate) skinned_gpu_cpu_morphed_previous_shape_velocity_missing_count: usize,
    pub(crate) skinned_gpu_skinning_draw_count: usize,
    pub(crate) skinned_gpu_velocity_draw_count: usize,
    pub(crate) indirect_draw_count: usize,
    pub(crate) lod_draw_count: usize,
    pub(crate) static_batch_candidate_group_count: usize,
    pub(crate) static_batch_candidate_draw_count: usize,
    pub(crate) dynamic_batch_candidate_group_count: usize,
    pub(crate) dynamic_batch_candidate_draw_count: usize,
    pub(crate) gpu_instancing_candidate_group_count: usize,
    pub(crate) gpu_instancing_candidate_draw_count: usize,
    pub(crate) indirect_batch_count: usize,
    pub(crate) indirect_batched_draw_count: usize,
    pub(crate) indirect_fallback_draw_count: usize,
    pub(crate) indirect_args_count: usize,
    pub(crate) gpu_scene_primitive_count: u32,
    pub(crate) gpu_scene_instance_count: u32,
    pub(crate) gpu_scene_dirty_entry_count: usize,
    pub(crate) gpu_scene_uploaded_bytes: u64,
    pub(crate) gpu_scene_upload_path: RenderGpuSceneUploadPath,
    pub(crate) gpu_scene_free_span_count: usize,
    pub(crate) gpu_scene_primitive_upload_range_count: usize,
    pub(crate) gpu_scene_instance_upload_range_count: usize,
    pub(crate) previous_velocity_transform_draw_count: usize,
    pub(crate) missing_velocity_transform_draw_count: usize,
    pub(crate) command_count: usize,
    pub(crate) depth_prepass_command_count: usize,
    pub(crate) shadow_command_count: usize,
    pub(crate) opaque_command_count: usize,
    pub(crate) alpha_mask_command_count: usize,
    pub(crate) transparent_command_count: usize,
    pub(crate) velocity_command_count: usize,
    pub(crate) taa_reactive_mask_command_count: usize,
    pub(crate) cached_command_hit_count: usize,
    pub(crate) command_rebuild_count: usize,
    pub(crate) dynamic_command_count: usize,
    pub(crate) cache_miss_count: usize,
    pub(crate) cache_invalidated_transform_count: usize,
    pub(crate) cache_invalidated_geometry_count: usize,
    pub(crate) cache_invalidated_material_count: usize,
    pub(crate) state_change_count: usize,
    pub(crate) bind_skip_count: usize,
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

impl PreparedMeshQueueStats {
    pub(crate) fn with_mesh_pass_command_buffer_stats(
        mut self,
        command_stats: MeshPassCommandBufferStats,
    ) -> Self {
        self.command_count = command_stats.command_count;
        self.depth_prepass_command_count = command_stats.depth_prepass_command_count;
        self.shadow_command_count = command_stats.shadow_command_count;
        self.opaque_command_count = command_stats.opaque_command_count;
        self.alpha_mask_command_count = command_stats.alpha_mask_command_count;
        self.transparent_command_count = command_stats.transparent_command_count;
        self.velocity_command_count = command_stats.velocity_command_count;
        self.taa_reactive_mask_command_count = command_stats.taa_reactive_mask_command_count;
        self.cached_command_hit_count = command_stats.cached_command_hit_count;
        self.command_rebuild_count = command_stats.command_rebuild_count;
        self.dynamic_command_count = command_stats.dynamic_command_count;
        self.cache_miss_count = command_stats.cache_miss_count;
        self.cache_invalidated_transform_count = command_stats.cache_invalidated_transform_count;
        self.cache_invalidated_geometry_count = command_stats.cache_invalidated_geometry_count;
        self.cache_invalidated_material_count = command_stats.cache_invalidated_material_count;
        self.indirect_batch_count = command_stats.indirect_batch_count;
        self.indirect_batched_draw_count = command_stats.indirect_batched_draw_count;
        self.indirect_fallback_draw_count = command_stats.indirect_fallback_draw_count;
        self.indirect_args_count = command_stats.indirect_args_count;
        self
    }

    pub(crate) fn with_mesh_draw_replay_stats(mut self, replay_stats: MeshDrawReplayStats) -> Self {
        self.state_change_count = replay_stats.state_change_count as usize;
        self.bind_skip_count = replay_stats.bind_skip_count as usize;
        self
    }

    pub(crate) fn with_gpu_scene_stats(
        mut self,
        stats: GpuSceneStats,
        upload_report: GpuSceneUploadReport,
    ) -> Self {
        self.gpu_scene_primitive_count = stats.primitive_count;
        self.gpu_scene_instance_count = stats.instance_count;
        self.gpu_scene_dirty_entry_count = stats.dirty_entry_count;
        self.gpu_scene_uploaded_bytes = upload_report.uploaded_bytes;
        self.gpu_scene_upload_path = render_gpu_scene_upload_path(upload_report.upload_path);
        self.gpu_scene_free_span_count = stats.free_span_count;
        self.gpu_scene_primitive_upload_range_count = upload_report.primitive_upload_range_count;
        self.gpu_scene_instance_upload_range_count = upload_report.instance_upload_range_count;
        self
    }
}

fn render_gpu_scene_upload_path(path: GpuSceneUploadPath) -> RenderGpuSceneUploadPath {
    match path {
        GpuSceneUploadPath::DirectQueueWrite => RenderGpuSceneUploadPath::DirectQueueWrite,
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
    K: Clone + Eq + Hash,
{
    let mut stats = PreparedMeshQueueStats::default();
    let mut static_batch_groups = HashMap::<K, usize>::new();
    let mut dynamic_batch_groups = HashMap::<K, usize>::new();
    let mut gpu_instancing_groups = HashMap::<K, usize>::new();

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
        match profile.geometry_source() {
            MeshDrawGeometrySource::Prepared => stats.prepared_geometry_draw_count += 1,
            MeshDrawGeometrySource::Dynamic | MeshDrawGeometrySource::DynamicGpuSkinningSource => {
                stats.dynamic_geometry_draw_count += 1
            }
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
    use crate::graphics::scene::scene_renderer::mesh::mesh_pass::{
        MeshDrawReplayStats, MeshPassCommandBufferStats,
    };

    #[test]
    fn prepared_queue_stats_allow_early_z_only_for_opaque_and_alpha_mask() {
        let stats = summarize_prepared_mesh_queue_items([
            item(
                profile(
                    MeshDrawQueuePhase::Opaque,
                    MeshDrawGeometrySource::Prepared,
                    Mobility::Static,
                    false,
                ),
                true,
                false,
                1_u8,
            ),
            item(
                profile(
                    MeshDrawQueuePhase::AlphaMask,
                    MeshDrawGeometrySource::Prepared,
                    Mobility::Static,
                    false,
                ),
                true,
                false,
                2,
            ),
            item(
                profile(
                    MeshDrawQueuePhase::Transparent,
                    MeshDrawGeometrySource::Prepared,
                    Mobility::Static,
                    false,
                ),
                false,
                false,
                3,
            ),
        ]);

        assert_eq!(stats.draw_count, 3);
        assert_eq!(stats.opaque_draw_count, 1);
        assert_eq!(stats.alpha_mask_draw_count, 1);
        assert_eq!(stats.transparent_draw_count, 1);
        assert_eq!(stats.early_z_draw_count, 2);
        assert_eq!(stats.shadow_caster_draw_count, 2);
        assert_eq!(stats.alpha_mask_shadow_caster_draw_count, 1);
    }

    #[test]
    fn prepared_queue_stats_filter_material_shadow_casters_without_changing_phase_counts() {
        let stats = summarize_prepared_mesh_queue_items([
            item(
                profile(
                    MeshDrawQueuePhase::Opaque,
                    MeshDrawGeometrySource::Prepared,
                    Mobility::Static,
                    false,
                ),
                true,
                false,
                1_u8,
            ),
            item(
                profile(
                    MeshDrawQueuePhase::Opaque,
                    MeshDrawGeometrySource::Prepared,
                    Mobility::Static,
                    false,
                ),
                false,
                false,
                2,
            ),
            item(
                profile(
                    MeshDrawQueuePhase::AlphaMask,
                    MeshDrawGeometrySource::Prepared,
                    Mobility::Static,
                    false,
                ),
                false,
                false,
                3,
            ),
        ]);

        assert_eq!(stats.draw_count, 3);
        assert_eq!(stats.opaque_draw_count, 2);
        assert_eq!(stats.alpha_mask_draw_count, 1);
        assert_eq!(stats.early_z_draw_count, 3);
        assert_eq!(stats.shadow_caster_draw_count, 1);
        assert_eq!(stats.alpha_mask_shadow_caster_draw_count, 0);
    }

    #[test]
    fn shadow_caster_phase_matches_early_z_phase_policy() {
        assert!(MeshDrawQueuePhase::Opaque.casts_shadow());
        assert!(MeshDrawQueuePhase::AlphaMask.casts_shadow());
        assert!(!MeshDrawQueuePhase::Transparent.casts_shadow());
    }

    #[test]
    fn prepared_queue_stats_require_repeated_direct_prepared_keys_for_batching() {
        let stats = summarize_prepared_mesh_queue_items([
            item(
                profile(
                    MeshDrawQueuePhase::Opaque,
                    MeshDrawGeometrySource::Prepared,
                    Mobility::Static,
                    false,
                ),
                true,
                false,
                "static-a",
            ),
            item(
                profile(
                    MeshDrawQueuePhase::Opaque,
                    MeshDrawGeometrySource::Prepared,
                    Mobility::Static,
                    false,
                ),
                true,
                false,
                "static-a",
            ),
            item(
                profile(
                    MeshDrawQueuePhase::Opaque,
                    MeshDrawGeometrySource::Prepared,
                    Mobility::Dynamic,
                    false,
                ),
                true,
                false,
                "dynamic-a",
            ),
            item(
                profile(
                    MeshDrawQueuePhase::Opaque,
                    MeshDrawGeometrySource::Prepared,
                    Mobility::Dynamic,
                    false,
                ),
                true,
                false,
                "dynamic-a",
            ),
            item(
                profile(
                    MeshDrawQueuePhase::Opaque,
                    MeshDrawGeometrySource::Dynamic,
                    Mobility::Dynamic,
                    false,
                ),
                true,
                false,
                "dynamic-a",
            ),
            item(
                profile(
                    MeshDrawQueuePhase::Opaque,
                    MeshDrawGeometrySource::Prepared,
                    Mobility::Static,
                    true,
                ),
                true,
                false,
                "static-a",
            ),
            item(
                profile(
                    MeshDrawQueuePhase::Transparent,
                    MeshDrawGeometrySource::Prepared,
                    Mobility::Static,
                    false,
                ),
                false,
                false,
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

    #[test]
    fn prepared_queue_stats_count_dynamic_velocity_history_readiness() {
        let stats = summarize_prepared_mesh_queue_items([
            item(
                profile(
                    MeshDrawQueuePhase::Opaque,
                    MeshDrawGeometrySource::Prepared,
                    Mobility::Static,
                    false,
                ),
                true,
                true,
                "static-with-history",
            ),
            item(
                profile(
                    MeshDrawQueuePhase::Opaque,
                    MeshDrawGeometrySource::Prepared,
                    Mobility::Dynamic,
                    false,
                ),
                true,
                true,
                "dynamic-opaque-ready",
            ),
            item(
                profile(
                    MeshDrawQueuePhase::AlphaMask,
                    MeshDrawGeometrySource::Prepared,
                    Mobility::Dynamic,
                    false,
                ),
                true,
                false,
                "dynamic-alpha-missing",
            ),
            item(
                profile(
                    MeshDrawQueuePhase::Transparent,
                    MeshDrawGeometrySource::Prepared,
                    Mobility::Dynamic,
                    false,
                ),
                false,
                true,
                "dynamic-transparent-ready",
            ),
        ]);

        assert_eq!(stats.previous_velocity_transform_draw_count, 2);
        assert_eq!(stats.missing_velocity_transform_draw_count, 1);
    }

    #[test]
    fn prepared_queue_stats_count_skinned_gpu_draws_separately_from_cpu_fallbacks() {
        let stats = summarize_prepared_mesh_queue_items([
            gpu_skinned_item(
                skinned_gpu_profile(
                    MeshDrawQueuePhase::Opaque,
                    MeshDrawGeometrySource::Prepared,
                    Mobility::Dynamic,
                    false,
                ),
                true,
                false,
                "gpu-skinned-prepared",
            ),
            skinned_without_palette_item(
                profile(
                    MeshDrawQueuePhase::Opaque,
                    MeshDrawGeometrySource::Dynamic,
                    Mobility::Dynamic,
                    false,
                ),
                true,
                true,
                "cpu-skinned-over-uniform-limit",
            ),
            item(
                profile(
                    MeshDrawQueuePhase::Opaque,
                    MeshDrawGeometrySource::Dynamic,
                    Mobility::Dynamic,
                    false,
                ),
                true,
                true,
                "morphed-dynamic",
            ),
            item(
                profile(
                    MeshDrawQueuePhase::Opaque,
                    MeshDrawGeometrySource::Prepared,
                    Mobility::Static,
                    false,
                ),
                true,
                false,
                "prepared-static",
            ),
        ]);

        assert_eq!(stats.skinned_draw_count, 2);
        assert_eq!(stats.skinned_palette_upload_count, 1);
        assert_eq!(stats.skinned_previous_palette_upload_count, 0);
        assert_eq!(stats.skinned_gpu_source_candidate_count, 1);
        assert_eq!(stats.skinned_gpu_cpu_morphed_source_candidate_count, 0);
        assert_eq!(stats.skinned_gpu_skinning_draw_count, 1);
        assert_eq!(stats.skinned_gpu_velocity_draw_count, 0);
        assert_eq!(stats.dynamic_geometry_draw_count, 2);
        assert_eq!(stats.prepared_geometry_draw_count, 2);
    }

    #[test]
    fn prepared_queue_stats_count_cpu_morphed_gpu_skinning_source_as_dynamic_geometry() {
        let stats = summarize_prepared_mesh_queue_items([cpu_morphed_gpu_skinned_item(
            skinned_gpu_profile(
                MeshDrawQueuePhase::Opaque,
                MeshDrawGeometrySource::DynamicGpuSkinningSource,
                Mobility::Dynamic,
                false,
            ),
            true,
            false,
            "cpu-morphed-gpu-skinned",
        )]);

        assert_eq!(stats.skinned_draw_count, 1);
        assert_eq!(stats.skinned_palette_upload_count, 1);
        assert_eq!(stats.skinned_gpu_source_candidate_count, 1);
        assert_eq!(stats.skinned_gpu_cpu_morphed_source_candidate_count, 1);
        assert_eq!(
            stats.skinned_gpu_cpu_morphed_previous_shape_velocity_missing_count,
            1
        );
        assert_eq!(stats.skinned_gpu_skinning_draw_count, 1);
        assert_eq!(stats.skinned_gpu_velocity_draw_count, 0);
        assert_eq!(stats.previous_velocity_transform_draw_count, 0);
        assert_eq!(stats.missing_velocity_transform_draw_count, 0);
        assert_eq!(stats.dynamic_geometry_draw_count, 1);
        assert_eq!(stats.prepared_geometry_draw_count, 0);
        assert_eq!(stats.dynamic_batch_candidate_group_count, 0);
        assert_eq!(stats.gpu_instancing_candidate_group_count, 0);
    }

    #[test]
    fn prepared_queue_stats_count_conventional_mesh_lod_draws() {
        let stats = summarize_prepared_mesh_queue_items([
            item(
                mesh_lod_profile(
                    MeshDrawQueuePhase::Opaque,
                    MeshDrawGeometrySource::Prepared,
                    Mobility::Static,
                    false,
                ),
                true,
                false,
                "lod-prepared",
            ),
            item(
                profile(
                    MeshDrawQueuePhase::Opaque,
                    MeshDrawGeometrySource::Prepared,
                    Mobility::Static,
                    false,
                ),
                true,
                false,
                "base-prepared",
            ),
        ]);

        assert_eq!(stats.lod_draw_count, 1);
        assert_eq!(stats.prepared_geometry_draw_count, 2);
        assert_eq!(stats.static_batch_candidate_group_count, 0);
    }

    #[test]
    fn prepared_queue_stats_count_gpu_skinned_velocity_with_previous_palette() {
        let stats = summarize_prepared_mesh_queue_items([gpu_skinned_item(
            skinned_gpu_profile(
                MeshDrawQueuePhase::Opaque,
                MeshDrawGeometrySource::Prepared,
                Mobility::Dynamic,
                false,
            ),
            true,
            true,
            "gpu-skinned-velocity",
        )]);

        assert_eq!(stats.skinned_previous_palette_upload_count, 1);
        assert_eq!(stats.skinned_gpu_velocity_draw_count, 1);
        assert_eq!(stats.previous_velocity_transform_draw_count, 1);
        assert_eq!(stats.missing_velocity_transform_draw_count, 0);
    }

    #[test]
    fn prepared_queue_stats_carry_mesh_pass_command_buffer_counts() {
        let stats = PreparedMeshQueueStats::default().with_mesh_pass_command_buffer_stats(
            MeshPassCommandBufferStats {
                command_count: 9,
                depth_prepass_command_count: 2,
                shadow_command_count: 1,
                opaque_command_count: 3,
                alpha_mask_command_count: 1,
                transparent_command_count: 1,
                velocity_command_count: 1,
                taa_reactive_mask_command_count: 1,
                direct_indexed_count: 7,
                indirect_indexed_count: 2,
                gpu_scene_instance_count: 9,
                cached_command_hit_count: 0,
                command_rebuild_count: 9,
                dynamic_command_count: 9,
                cache_miss_count: 3,
                cache_invalidated_transform_count: 0,
                cache_invalidated_geometry_count: 1,
                cache_invalidated_material_count: 2,
                indirect_batch_count: 2,
                indirect_batched_draw_count: 5,
                indirect_fallback_draw_count: 4,
                indirect_args_count: 5,
            },
        );

        assert_eq!(stats.command_count, 9);
        assert_eq!(stats.depth_prepass_command_count, 2);
        assert_eq!(stats.shadow_command_count, 1);
        assert_eq!(stats.opaque_command_count, 3);
        assert_eq!(stats.alpha_mask_command_count, 1);
        assert_eq!(stats.transparent_command_count, 1);
        assert_eq!(stats.velocity_command_count, 1);
        assert_eq!(stats.taa_reactive_mask_command_count, 1);
        assert_eq!(stats.cached_command_hit_count, 0);
        assert_eq!(stats.command_rebuild_count, 9);
        assert_eq!(stats.dynamic_command_count, 9);
        assert_eq!(stats.cache_miss_count, 3);
        assert_eq!(stats.cache_invalidated_transform_count, 0);
        assert_eq!(stats.cache_invalidated_geometry_count, 1);
        assert_eq!(stats.cache_invalidated_material_count, 2);
        assert_eq!(stats.indirect_batch_count, 2);
        assert_eq!(stats.indirect_batched_draw_count, 5);
        assert_eq!(stats.indirect_fallback_draw_count, 4);
        assert_eq!(stats.indirect_args_count, 5);
        assert_eq!(stats.state_change_count, 0);
        assert_eq!(stats.bind_skip_count, 0);
    }

    #[test]
    fn prepared_queue_stats_carry_mesh_draw_replay_counts() {
        let stats =
            PreparedMeshQueueStats::default().with_mesh_draw_replay_stats(MeshDrawReplayStats {
                draw_call_count: 12,
                state_change_count: 4,
                bind_skip_count: 7,
            });

        assert_eq!(stats.state_change_count, 4);
        assert_eq!(stats.bind_skip_count, 7);
    }

    #[test]
    fn prepared_queue_stats_carry_gpu_scene_counts() {
        let stats = PreparedMeshQueueStats::default().with_gpu_scene_stats(
            GpuSceneStats {
                primitive_count: 5,
                instance_count: 7,
                light_count: 11,
                dirty_entry_count: 3,
                uploaded_bytes: 99,
                primitive_capacity: 1024,
                instance_capacity: 2048,
                light_capacity: 256,
                free_span_count: 2,
            },
            GpuSceneUploadReport {
                upload_path: GpuSceneUploadPath::DirectQueueWrite,
                uploaded_bytes: 88,
                primitive_upload_range_count: 4,
                instance_upload_range_count: 6,
                ..GpuSceneUploadReport::default()
            },
        );

        assert_eq!(stats.gpu_scene_primitive_count, 5);
        assert_eq!(stats.gpu_scene_instance_count, 7);
        assert_eq!(stats.gpu_scene_dirty_entry_count, 3);
        assert_eq!(stats.gpu_scene_uploaded_bytes, 88);
        assert_eq!(
            stats.gpu_scene_upload_path,
            RenderGpuSceneUploadPath::DirectQueueWrite
        );
        assert_eq!(stats.gpu_scene_free_span_count, 2);
        assert_eq!(stats.gpu_scene_primitive_upload_range_count, 4);
        assert_eq!(stats.gpu_scene_instance_upload_range_count, 6);
    }

    #[test]
    fn prepared_queue_stats_exclude_gpu_skinned_draws_from_direct_batch_candidates() {
        let stats = summarize_prepared_mesh_queue_items([
            gpu_skinned_item(
                skinned_gpu_profile(
                    MeshDrawQueuePhase::Opaque,
                    MeshDrawGeometrySource::Prepared,
                    Mobility::Dynamic,
                    false,
                ),
                true,
                false,
                "gpu-skinned-shared-key",
            ),
            gpu_skinned_item(
                skinned_gpu_profile(
                    MeshDrawQueuePhase::Opaque,
                    MeshDrawGeometrySource::Prepared,
                    Mobility::Dynamic,
                    false,
                ),
                true,
                false,
                "gpu-skinned-shared-key",
            ),
        ]);

        assert_eq!(stats.prepared_geometry_draw_count, 2);
        assert_eq!(stats.skinned_gpu_skinning_draw_count, 2);
        assert_eq!(stats.dynamic_batch_candidate_group_count, 0);
        assert_eq!(stats.dynamic_batch_candidate_draw_count, 0);
        assert_eq!(stats.gpu_instancing_candidate_group_count, 0);
        assert_eq!(stats.gpu_instancing_candidate_draw_count, 0);
    }

    fn item<K>(
        profile: MeshDrawQueueProfile,
        casts_shadow: bool,
        has_previous_velocity_transform: bool,
        key: K,
    ) -> (
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
    ) {
        (
            profile,
            casts_shadow,
            has_previous_velocity_transform,
            false,
            false,
            false,
            false,
            false,
            false,
            key,
        )
    }

    fn gpu_skinned_item<K>(
        profile: MeshDrawQueueProfile,
        casts_shadow: bool,
        has_previous_velocity_transform: bool,
        key: K,
    ) -> (
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
    ) {
        (
            profile,
            casts_shadow,
            has_previous_velocity_transform,
            true,
            true,
            has_previous_velocity_transform,
            true,
            false,
            true,
            key,
        )
    }

    fn cpu_morphed_gpu_skinned_item<K>(
        profile: MeshDrawQueueProfile,
        casts_shadow: bool,
        has_previous_velocity_transform: bool,
        key: K,
    ) -> (
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
    ) {
        (
            profile,
            casts_shadow,
            has_previous_velocity_transform,
            true,
            true,
            has_previous_velocity_transform,
            true,
            true,
            true,
            key,
        )
    }

    fn skinned_without_palette_item<K>(
        profile: MeshDrawQueueProfile,
        casts_shadow: bool,
        has_previous_velocity_transform: bool,
        key: K,
    ) -> (
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
    ) {
        (
            profile,
            casts_shadow,
            has_previous_velocity_transform,
            true,
            false,
            false,
            false,
            false,
            false,
            key,
        )
    }

    fn profile(
        phase: MeshDrawQueuePhase,
        geometry_source: MeshDrawGeometrySource,
        mobility: Mobility,
        uses_indirect_draw: bool,
    ) -> MeshDrawQueueProfile {
        MeshDrawQueueProfile::new(
            phase,
            geometry_source,
            mobility,
            uses_indirect_draw,
            false,
            false,
        )
    }

    fn skinned_gpu_profile(
        phase: MeshDrawQueuePhase,
        geometry_source: MeshDrawGeometrySource,
        mobility: Mobility,
        uses_indirect_draw: bool,
    ) -> MeshDrawQueueProfile {
        MeshDrawQueueProfile::new(
            phase,
            geometry_source,
            mobility,
            uses_indirect_draw,
            true,
            false,
        )
    }

    fn mesh_lod_profile(
        phase: MeshDrawQueuePhase,
        geometry_source: MeshDrawGeometrySource,
        mobility: Mobility,
        uses_indirect_draw: bool,
    ) -> MeshDrawQueueProfile {
        MeshDrawQueueProfile::new(
            phase,
            geometry_source,
            mobility,
            uses_indirect_draw,
            false,
            true,
        )
    }
}

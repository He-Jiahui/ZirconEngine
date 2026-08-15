use zircon_runtime::core::framework::render::RenderHybridGiGlobalSdfStats;

use crate::hybrid_gi::renderer::{GlobalSdfGpuBuildStats, GlobalSdfGpuState};
use crate::hybrid_gi::scene_representation::HybridGiGlobalSdfSceneState;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct GlobalSdfCpuPrepareTimings {
    pub(super) mesh_object_collection_time_us: u64,
    pub(super) mesh_scene_sync_time_us: u64,
    pub(super) global_sdf_residency_time_us: u64,
    pub(super) global_sdf_influence_update_time_us: u64,
    pub(super) global_sdf_candidate_build_time_us: u64,
    pub(super) mesh_projection_cache_hit: bool,
}

impl GlobalSdfCpuPrepareTimings {
    pub(super) fn total_time_us(self) -> u64 {
        self.mesh_object_collection_time_us
            .saturating_add(self.mesh_scene_sync_time_us)
            .saturating_add(self.global_sdf_residency_time_us)
            .saturating_add(self.global_sdf_influence_update_time_us)
            .saturating_add(self.global_sdf_candidate_build_time_us)
    }
}

pub(super) fn global_sdf_runtime_stats(
    scene: &HybridGiGlobalSdfSceneState,
    gpu_state: &GlobalSdfGpuState,
    timings: GlobalSdfCpuPrepareTimings,
    object_count: usize,
    uploaded_page_count: usize,
    build_stats: GlobalSdfGpuBuildStats,
) -> RenderHybridGiGlobalSdfStats {
    RenderHybridGiGlobalSdfStats {
        cpu_prepare_time_us: timings.total_time_us(),
        cpu_mesh_object_collection_time_us: timings.mesh_object_collection_time_us,
        cpu_mesh_scene_sync_time_us: timings.mesh_scene_sync_time_us,
        cpu_residency_time_us: timings.global_sdf_residency_time_us,
        cpu_influence_update_time_us: timings.global_sdf_influence_update_time_us,
        cpu_candidate_build_time_us: timings.global_sdf_candidate_build_time_us,
        mesh_projection_cache_hit: timings.mesh_projection_cache_hit,
        object_count,
        resident_page_count: scene.resident_page_count(),
        sampleable_page_count: scene.sampleable_page_count(),
        dirty_page_count: scene.dirty_page_count(),
        dispatched_page_count: build_stats.dispatched_page_count,
        uploaded_page_count,
        deferred_page_count: build_stats.deferred_page_count,
        candidate_overflow_page_count: build_stats.candidate_overflow_page_count,
        candidate_contributor_count: scene.candidate_contributor_count(),
        clipmap_fallback_count: scene.clipmap_fallback_count(),
        candidate_bucket_capacity_bytes: scene.candidate_bucket_capacity_bytes(),
        persistent_resource_byte_count: gpu_state.persistent_resource_byte_count(),
        transient_buffer_creation_count: build_stats.transient_buffer_creation_count,
        transient_bind_group_creation_count: build_stats.transient_bind_group_creation_count,
        transient_parameter_upload_byte_count: build_stats.transient_parameter_upload_byte_count,
        transient_page_upload_byte_count: build_stats.transient_page_upload_byte_count,
        transient_mesh_upload_byte_count: build_stats.transient_mesh_upload_byte_count,
        transient_completion_upload_byte_count: build_stats.transient_completion_upload_byte_count,
        transient_upload_byte_count: build_stats.transient_upload_byte_count,
    }
}

#[cfg(test)]
mod tests {
    use super::GlobalSdfCpuPrepareTimings;

    #[test]
    fn cpu_prepare_timings_sum_all_named_phases_without_overflow() {
        let timings = GlobalSdfCpuPrepareTimings {
            mesh_object_collection_time_us: u64::MAX,
            mesh_scene_sync_time_us: 1,
            global_sdf_residency_time_us: 1,
            global_sdf_influence_update_time_us: 1,
            global_sdf_candidate_build_time_us: 1,
            mesh_projection_cache_hit: false,
        };

        assert_eq!(timings.total_time_us(), u64::MAX);
    }
}

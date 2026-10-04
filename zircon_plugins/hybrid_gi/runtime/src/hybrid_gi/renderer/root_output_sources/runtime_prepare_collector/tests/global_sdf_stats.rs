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

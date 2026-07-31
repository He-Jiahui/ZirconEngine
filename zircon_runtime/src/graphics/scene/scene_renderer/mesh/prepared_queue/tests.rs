use super::*;
use crate::core::framework::scene::Mobility;

mod gpu_sources;

#[test]
fn prepared_queue_candidate_stats_share_one_keyed_group_table() {
    let source = include_str!("../prepared_queue.rs");

    assert!(source.contains("HashMap::<K, CandidateGroupCounts>::new()"));
    assert!(!source.contains("static_batch_groups = HashMap"));
    assert!(!source.contains("dynamic_batch_groups = HashMap"));
    assert!(!source.contains("gpu_instancing_groups = HashMap"));
}

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
            MeshDrawGeometrySource::DynamicCpuMorphedGpuSkinningSource,
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
fn prepared_queue_stats_count_direct_cpu_morphed_source_as_dynamic_geometry() {
    let stats = summarize_prepared_mesh_queue_items([item(
        profile(
            MeshDrawQueuePhase::Opaque,
            MeshDrawGeometrySource::DynamicCpuMorphedSource,
            Mobility::Dynamic,
            false,
        ),
        true,
        false,
        "direct-cpu-morphed",
    )]);

    assert_eq!(stats.dynamic_geometry_draw_count, 1);
    assert_eq!(stats.gpu_morphed_source_draw_count, 0);
    assert_eq!(stats.gpu_skinned_morphed_source_draw_count, 0);
    assert_eq!(stats.prepared_geometry_draw_count, 0);
    assert_eq!(stats.skinned_gpu_skinning_draw_count, 0);
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

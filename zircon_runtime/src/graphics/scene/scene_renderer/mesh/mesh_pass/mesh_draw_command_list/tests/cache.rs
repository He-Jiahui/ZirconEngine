use super::*;

#[test]
fn mesh_pass_command_buffers_reuse_static_cached_commands_on_second_frame() {
    let mut variants = MeshPipelineVariantRegistry::default();
    let mut cache = CachedMeshDrawCommands::default();
    let static_state = RenderMeshStaticState::new(true, 11, 17);
    let batch = static_batch(MeshDrawQueuePhase::Opaque, 10)
        .with_cache_identity(7, 0)
        .with_static_state(static_state)
        .with_casts_shadow(true)
        .with_taa_reactive_mask_strength(0.75);

    let first = build_mesh_pass_command_buffers_from_batches_cached(
        [batch.clone()],
        &mut variants,
        &mut cache,
        1,
        ShaderQualityTier::default(),
    )
    .stats();
    let second = build_mesh_pass_command_buffers_from_batches_cached(
        [batch],
        &mut variants,
        &mut cache,
        2,
        ShaderQualityTier::default(),
    )
    .stats();

    assert_eq!(first.command_count, 4);
    assert_eq!(first.cached_command_hit_count, 0);
    assert_eq!(first.command_rebuild_count, 4);
    assert_eq!(first.dynamic_command_count, 1);
    assert_eq!(first.cache_miss_count, 3);
    assert_eq!(first.cache_invalidated_material_count, 0);
    assert_eq!(second.command_count, 4);
    assert_eq!(second.cached_command_hit_count, 3);
    assert_eq!(second.command_rebuild_count, 1);
    assert_eq!(second.dynamic_command_count, 1);
    assert_eq!(second.cache_miss_count, 0);
    assert_eq!(second.cache_invalidated_material_count, 0);
}

#[test]
fn mesh_pass_command_buffers_report_static_cache_invalidation_reasons() {
    let mut variants = MeshPipelineVariantRegistry::default();
    let mut cache = CachedMeshDrawCommands::default();
    let static_state = RenderMeshStaticState::new(true, 11, 17);
    let changed_material = RenderMeshStaticState::new(true, 11, 23);
    let batch = static_batch(MeshDrawQueuePhase::Opaque, 10)
        .with_cache_identity(7, 0)
        .with_static_state(static_state)
        .with_casts_shadow(true);

    let _ = build_mesh_pass_command_buffers_from_batches_cached(
        [batch],
        &mut variants,
        &mut cache,
        1,
        ShaderQualityTier::default(),
    );
    let changed = build_mesh_pass_command_buffers_from_batches_cached(
        [static_batch(MeshDrawQueuePhase::Opaque, 10)
            .with_cache_identity(7, 0)
            .with_static_state(changed_material)
            .with_casts_shadow(true)],
        &mut variants,
        &mut cache,
        2,
        ShaderQualityTier::default(),
    )
    .stats();

    assert_eq!(changed.cached_command_hit_count, 0);
    assert_eq!(changed.cache_miss_count, 0);
    assert_eq!(changed.cache_invalidated_transform_count, 0);
    assert_eq!(changed.cache_invalidated_geometry_count, 0);
    assert_eq!(changed.cache_invalidated_material_count, 3);
    assert_eq!(changed.command_rebuild_count, 3);
}

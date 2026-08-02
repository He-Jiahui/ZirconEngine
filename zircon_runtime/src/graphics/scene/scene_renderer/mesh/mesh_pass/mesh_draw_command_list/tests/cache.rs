use super::*;

#[test]
fn mesh_pass_command_buffers_reuse_static_cached_commands_on_second_frame() {
    let mut variants = MeshPipelineVariantRegistry::default();
    let mut cache = CachedMeshDrawCommands::default();
    let static_state = RenderMeshStaticState::new(true, 11, 17);
    let batch = static_batch(MeshDrawQueuePhase::Opaque, 10)
        .with_cache_identity(7, 7 << 16, 0)
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
        .with_cache_identity(7, 7 << 16, 0)
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
            .with_cache_identity(7, 7 << 16, 0)
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

#[test]
fn render_perf_parallel_prepare_deterministic_sort() {
    let task_pool = TaskPool::new(TaskPoolDescriptor::compute().with_worker_threads(4));
    let batches = [
        static_batch(MeshDrawQueuePhase::Opaque, 30)
            .with_source_draw_index(0)
            .with_cache_identity(7, 7 << 16, 0)
            .with_static_state(RenderMeshStaticState::new(true, 11, 17))
            .with_casts_shadow(true)
            .with_taa_reactive_mask_strength(0.25),
        batch(MeshDrawQueuePhase::Transparent, 20).with_source_draw_index(1),
        static_batch(MeshDrawQueuePhase::AlphaMask, 10)
            .with_source_draw_index(2)
            .with_cache_identity(9, 9 << 16, 0)
            .with_static_state(RenderMeshStaticState::new(true, 13, 19))
            .with_casts_shadow(true),
        batch(MeshDrawQueuePhase::Opaque, 40)
            .with_source_draw_index(3)
            .with_previous_velocity_transform(true),
    ];
    let mut serial_variants = MeshPipelineVariantRegistry::default();
    let mut parallel_variants = MeshPipelineVariantRegistry::default();
    let mut serial_cache = CachedMeshDrawCommands::default();
    let mut parallel_cache = CachedMeshDrawCommands::default();

    for generation in [1, 2] {
        let serial = build_mesh_pass_command_buffers_from_batches_cached(
            batches.iter().cloned(),
            &mut serial_variants,
            &mut serial_cache,
            generation,
            ShaderQualityTier::High,
        );
        let parallel = build_mesh_pass_command_buffers_from_batches_cached_parallel(
            batches.iter().cloned(),
            &mut parallel_variants,
            &mut parallel_cache,
            generation,
            ShaderQualityTier::High,
            &task_pool,
        );

        assert_eq!(parallel.stats(), serial.stats());
        assert_eq!(command_signatures(&parallel), command_signatures(&serial));
    }
}

fn command_signatures(
    buffers: &MeshPassCommandBuffers,
) -> Vec<(usize, RenderPhase, MeshPassPipelineKind, u32, u64)> {
    [
        buffers.depth_prepass(),
        buffers.shadow(),
        buffers.opaque(),
        buffers.alpha_mask(),
        buffers.advanced_pbr_opaque(),
        buffers.transmission(),
        buffers.transparent(),
        buffers.velocity(),
        buffers.taa_reactive_mask(),
    ]
    .into_iter()
    .flat_map(|commands| commands.commands())
    .map(|command| {
        (
            command.source_draw_index,
            command.phase,
            command.pipeline_kind,
            command.pipeline_variant_id.value(),
            command.sort_key,
        )
    })
    .collect()
}

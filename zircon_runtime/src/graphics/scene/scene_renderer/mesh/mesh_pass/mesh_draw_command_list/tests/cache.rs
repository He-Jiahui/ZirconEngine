use std::sync::Arc;

use super::super::builder::should_prepare_batches_in_parallel;
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

    let first = build_mesh_pass_command_buffers_from_batches_cached(
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
    assert!(!Arc::ptr_eq(
        &first.opaque().commands()[0].static_payload(),
        &changed.opaque().commands()[0].static_payload()
    ));
}

#[test]
fn cached_static_commands_rebuild_for_changed_shader_quality() {
    let mut variants = MeshPipelineVariantRegistry::default();
    let mut cache = CachedMeshDrawCommands::default();
    let batch = static_batch(MeshDrawQueuePhase::Opaque, 10)
        .with_cache_identity(7, 7 << 16, 0)
        .with_static_state(RenderMeshStaticState::new(true, 11, 17));

    let low = build_mesh_pass_command_buffers_from_batches_cached(
        [batch.clone()],
        &mut variants,
        &mut cache,
        1,
        ShaderQualityTier::Low,
    );
    let high = build_mesh_pass_command_buffers_from_batches_cached(
        [batch],
        &mut variants,
        &mut cache,
        2,
        ShaderQualityTier::High,
    );

    assert_eq!(high.stats().cached_command_hit_count, 0);
    assert_eq!(high.stats().cache_miss_count, 2);
    assert_eq!(high.stats().command_rebuild_count, 2);
    assert_ne!(
        low.opaque().commands()[0].pipeline_variant_id,
        high.opaque().commands()[0].pipeline_variant_id
    );
}

#[test]
fn cached_static_command_hit_reprojects_current_batch_in_serial_path() {
    let mut variants = MeshPipelineVariantRegistry::default();
    let mut cache = CachedMeshDrawCommands::default();
    let initial = static_batch(MeshDrawQueuePhase::Opaque, 10)
        .with_source_draw_index(0)
        .with_cache_identity(7, 7 << 16, 0)
        .with_static_state(RenderMeshStaticState::new(true, 11, 17));
    let current = initial
        .clone()
        .with_source_draw_index(9)
        .with_sort_components(RenderPhaseSortComponents::new(70.0, 70))
        .with_gpu_scene_instance_span(31, 3);

    let first = build_mesh_pass_command_buffers_from_batches_cached(
        [initial],
        &mut variants,
        &mut cache,
        1,
        ShaderQualityTier::High,
    );
    let second = build_mesh_pass_command_buffers_from_batches_cached(
        [current.clone()],
        &mut variants,
        &mut cache,
        2,
        ShaderQualityTier::High,
    );

    assert_eq!(second.stats().cached_command_hit_count, 2);
    assert_eq!(second.stats().command_rebuild_count, 0);
    let first_command = &first.opaque().commands()[0];
    let second_command = &second.opaque().commands()[0];
    assert!(Arc::ptr_eq(
        &first_command.static_payload(),
        &second_command.static_payload()
    ));
    assert_current_batch_projection(second_command, &current);
}

#[test]
fn cached_static_command_hit_reprojects_current_batch_in_parallel_path() {
    let task_pool = TaskPool::new(TaskPoolDescriptor::compute().with_worker_threads(4));
    let mut variants = MeshPipelineVariantRegistry::default();
    let mut cache = CachedMeshDrawCommands::default();
    let initial = static_batch(MeshDrawQueuePhase::Opaque, 10)
        .with_source_draw_index(0)
        .with_cache_identity(7, 7 << 16, 0)
        .with_static_state(RenderMeshStaticState::new(true, 11, 17));
    let current = initial
        .clone()
        .with_source_draw_index(19)
        .with_sort_components(RenderPhaseSortComponents::new(70.0, 70))
        .with_gpu_scene_instance_span(31, 3);
    let other_initial = static_batch(MeshDrawQueuePhase::Opaque, 20)
        .with_source_draw_index(1)
        .with_cache_identity(9, 9 << 16, 0)
        .with_static_state(RenderMeshStaticState::new(true, 13, 19));
    let other_current = other_initial.clone().with_source_draw_index(20);

    let _ = build_mesh_pass_command_buffers_from_batches_cached_parallel(
        [initial, other_initial],
        &mut variants,
        &mut cache,
        1,
        ShaderQualityTier::High,
        &task_pool,
    );
    assert!(should_prepare_batches_in_parallel(
        &[current.clone(), other_current.clone()],
        &task_pool,
    ));
    let second = build_mesh_pass_command_buffers_from_batches_cached_parallel(
        [current.clone(), other_current],
        &mut variants,
        &mut cache,
        2,
        ShaderQualityTier::High,
        &task_pool,
    );

    assert_eq!(second.stats().cached_command_hit_count, 4);
    assert_eq!(second.stats().command_rebuild_count, 0);
    let command = second
        .opaque()
        .commands()
        .iter()
        .find(|command| command.source_draw_index == current.source_draw_index)
        .expect("projected command for the current static batch");
    assert_current_batch_projection(command, &current);
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

#[test]
fn render_parallel_prepare_normalizes_source_order_before_owner_transactions() {
    let task_pool = TaskPool::new(TaskPoolDescriptor::compute().with_worker_threads(4));
    let mut source_first = static_batch(MeshDrawQueuePhase::Opaque, 10)
        .with_source_draw_index(0)
        .with_cache_identity(7, 7 << 16, 0)
        .with_static_state(RenderMeshStaticState::new(true, 11, 17));
    source_first.pipeline_key.unlit = true;
    let source_second = static_batch(MeshDrawQueuePhase::Opaque, 20)
        .with_source_draw_index(1)
        .with_cache_identity(9, 9 << 16, 0)
        .with_static_state(RenderMeshStaticState::new(true, 13, 19));
    let batches = [source_second, source_first];
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

        assert_eq!(serial.stats(), parallel.stats());
        assert_eq!(command_signatures(&serial), command_signatures(&parallel));
    }
}

#[test]
fn render_parallel_prepare_duplicate_cache_keys_falls_back_to_serial_owner_path() {
    let task_pool = TaskPool::new(TaskPoolDescriptor::compute().with_worker_threads(4));
    let static_state = RenderMeshStaticState::new(true, 11, 17);
    let batches = [
        static_batch(MeshDrawQueuePhase::Opaque, 10)
            .with_source_draw_index(0)
            .with_cache_identity(7, 7 << 16, 0)
            .with_static_state(static_state),
        static_batch(MeshDrawQueuePhase::Opaque, 20)
            .with_source_draw_index(1)
            .with_cache_identity(7, 7 << 16, 0)
            .with_static_state(static_state),
    ];
    assert!(!should_prepare_batches_in_parallel(&batches, &task_pool));
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

#[test]
fn render_parallel_prepare_predicate_requires_multiple_workers_and_batches() {
    let parallel_pool = TaskPool::new(TaskPoolDescriptor::compute().with_worker_threads(4));
    let single_worker_pool = TaskPool::new(TaskPoolDescriptor::compute().with_worker_threads(1));
    let static_state = RenderMeshStaticState::new(true, 11, 17);
    let first = static_batch(MeshDrawQueuePhase::Opaque, 10)
        .with_source_draw_index(0)
        .with_cache_identity(7, 7 << 16, 0)
        .with_static_state(static_state);
    let second = static_batch(MeshDrawQueuePhase::Opaque, 20)
        .with_source_draw_index(1)
        .with_cache_identity(9, 9 << 16, 0)
        .with_static_state(static_state);

    assert!(!should_prepare_batches_in_parallel(
        &[first.clone(), second.clone()],
        &single_worker_pool,
    ));
    assert!(!should_prepare_batches_in_parallel(
        &[first.clone()],
        &parallel_pool,
    ));
    assert!(should_prepare_batches_in_parallel(
        &[first, second],
        &parallel_pool,
    ));
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

fn assert_current_batch_projection(command: &MeshDrawCommand, batch: &MeshBatchRef) {
    let expected = batch.command(
        command.phase,
        command.pipeline_kind,
        command.pipeline_variant_id,
    );

    assert_eq!(command.sort_key, expected.sort_key);
    assert_eq!(command.source_draw_index, expected.source_draw_index);
    assert_eq!(command.source_entity, expected.source_entity);
    match (&command.instance_source, &expected.instance_source) {
        (
            DrawInstanceSource::GpuSceneInstance {
                first_instance_index: actual_first_instance,
                instance_count: actual_instance_count,
            },
            DrawInstanceSource::GpuSceneInstance {
                first_instance_index: expected_first_instance,
                instance_count: expected_instance_count,
            },
        ) => {
            assert_eq!(actual_first_instance, expected_first_instance);
            assert_eq!(actual_instance_count, expected_instance_count);
        }
    }
    match (&command.draw_args, &expected.draw_args) {
        (
            MeshDrawArgs::DirectIndexed {
                first_index: actual_first_index,
                index_count: actual_index_count,
                first_instance: actual_first_instance,
                instance_count: actual_instance_count,
            },
            MeshDrawArgs::DirectIndexed {
                first_index: expected_first_index,
                index_count: expected_index_count,
                first_instance: expected_first_instance,
                instance_count: expected_instance_count,
            },
        ) => {
            assert_eq!(actual_first_index, expected_first_index);
            assert_eq!(actual_index_count, expected_index_count);
            assert_eq!(actual_first_instance, expected_first_instance);
            assert_eq!(actual_instance_count, expected_instance_count);
        }
        _ => panic!("expected projected direct indexed draw arguments"),
    }
}

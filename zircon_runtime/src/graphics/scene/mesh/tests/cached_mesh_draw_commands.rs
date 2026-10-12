use crate::core::framework::render::{
    RenderMeshStaticState, RenderPhase, RenderPhaseSortComponents,
};
use crate::core::framework::scene::Mobility;
use crate::graphics::scene::resources::default_pipeline_key;
use crate::graphics::scene::scene_renderer::mesh::mesh_draw::{
    MeshDrawGeometrySource, MeshDrawQueuePhase, MeshDrawQueueProfile,
};
use crate::graphics::scene::scene_renderer::mesh::mesh_pass::{
    DrawInstanceSource, MeshBatchRef, MeshBindHandle, MeshDrawArgs, MeshDrawCommand,
    MeshGeometryHandle, MeshPassPipelineKind, MeshPipelineVariantId,
};

use super::{CachedMeshDrawCommands, CachedMeshDrawKey};

#[test]
fn cached_mesh_draw_commands_reuse_matching_static_state() {
    let mut cache = CachedMeshDrawCommands::default();
    let key = CachedMeshDrawKey {
        stable_instance_key: (7 << 16) | 2,
        draw_ordinal: 2,
        phase: RenderPhase::Opaque3d,
        disabled_passes: Default::default(),
        shader_quality: Default::default(),
    };
    let state = RenderMeshStaticState::new(true, 11, 17);
    let command = test_command(RenderPhase::Opaque3d, 1);

    cache.store(key, &state, command.static_payload(), 3);
    assert!(cache.pins_pipeline_variant(command.pipeline_variant_id));

    let hit = cache
        .lookup(&key, &state, 4)
        .expect("matching state should hit");

    assert_eq!(hit.phase, command.phase);
    cache.retain_generation(4);
    assert_eq!(cache.len(), 1);
    assert!(cache.pins_pipeline_variant(command.pipeline_variant_id));
}

#[test]
fn cached_mesh_draw_commands_clear_all_static_payloads() {
    let mut cache = CachedMeshDrawCommands::default();
    let key = CachedMeshDrawKey {
        stable_instance_key: (7 << 16) | 2,
        draw_ordinal: 2,
        phase: RenderPhase::Opaque3d,
        disabled_passes: Default::default(),
        shader_quality: Default::default(),
    };
    let state = RenderMeshStaticState::new(true, 11, 17);
    let command = test_command(RenderPhase::Opaque3d, 1);
    cache.store(key, &state, command.static_payload(), 3);
    assert!(cache.pins_pipeline_variant(command.pipeline_variant_id));

    cache.clear();

    assert_eq!(cache.len(), 0);
    assert!(!cache.pins_pipeline_variant(command.pipeline_variant_id));
    assert!(cache.lookup(&key, &state, 4).is_none());
}

#[test]
fn cached_mesh_draw_commands_invalidate_changed_material_revision() {
    let mut cache = CachedMeshDrawCommands::default();
    let key = CachedMeshDrawKey {
        stable_instance_key: (7 << 16) | 2,
        draw_ordinal: 2,
        phase: RenderPhase::Opaque3d,
        disabled_passes: Default::default(),
        shader_quality: Default::default(),
    };
    let state = RenderMeshStaticState::new(true, 11, 17);
    let changed_material = RenderMeshStaticState::new(true, 11, 23);

    cache.store(
        key,
        &state,
        test_command(RenderPhase::Opaque3d, 1).static_payload(),
        1,
    );

    assert!(cache.lookup(&key, &changed_material, 2).is_none());
    cache.retain_generation(2);
    assert_eq!(cache.len(), 0);
    assert!(!cache.pins_pipeline_variant(MeshPipelineVariantId::new(1)));
}

#[test]
fn cached_mesh_draw_commands_reject_dynamic_transparent_and_indirect_batches() {
    let static_state = RenderMeshStaticState::new(true, 11, 17);
    let opaque_static = batch(MeshDrawQueuePhase::Opaque, Mobility::Static, false)
        .with_cache_identity(7, 7 << 16, 0)
        .with_static_state(static_state);
    let transparent_static = batch(MeshDrawQueuePhase::Transparent, Mobility::Static, false)
        .with_cache_identity(7, 7 << 16, 0)
        .with_static_state(static_state);
    let indirect_static = batch(MeshDrawQueuePhase::Opaque, Mobility::Static, true)
        .with_cache_identity(7, 7 << 16, 0)
        .with_static_state(static_state);
    let dynamic = batch(MeshDrawQueuePhase::Opaque, Mobility::Dynamic, false)
        .with_cache_identity(7, 7 << 16, 0)
        .with_static_state(static_state);

    assert!(CachedMeshDrawCommands::is_cacheable_batch_phase(
        &opaque_static,
        RenderPhase::Opaque3d
    ));
    assert!(!CachedMeshDrawCommands::is_cacheable_batch_phase(
        &transparent_static,
        RenderPhase::Transparent3d
    ));
    assert!(!CachedMeshDrawCommands::is_cacheable_batch_phase(
        &indirect_static,
        RenderPhase::Opaque3d
    ));
    assert!(!CachedMeshDrawCommands::is_cacheable_batch_phase(
        &dynamic,
        RenderPhase::Opaque3d
    ));
}

#[test]
#[should_panic(expected = "cached mesh draw payloads must use direct indexed topology")]
fn cached_mesh_draw_commands_reject_indirect_payload_storage() {
    let mut cache = CachedMeshDrawCommands::default();
    let key = CachedMeshDrawKey {
        stable_instance_key: 7 << 16,
        draw_ordinal: 0,
        phase: RenderPhase::Opaque3d,
        disabled_passes: Default::default(),
        shader_quality: Default::default(),
    };
    let state = RenderMeshStaticState::new(true, 11, 17);
    let command = MeshDrawCommand::new(
        RenderPhase::Opaque3d,
        MeshPassPipelineKind::Base,
        default_pipeline_key(),
        MeshPipelineVariantId::new(1),
        1,
        DrawInstanceSource::GpuSceneInstance {
            first_instance_index: 1,
            instance_count: 1,
        },
        MeshGeometryHandle::test(1),
        MeshDrawArgs::test_indexed_indirect(91, 0),
    );

    cache.store(key, &state, command.static_payload(), 1);
}

#[test]
fn cached_mesh_draw_commands_keep_sibling_primitives_separate() {
    let mut cache = CachedMeshDrawCommands::default();
    let state = RenderMeshStaticState::new(true, 11, 17);
    let first = CachedMeshDrawKey {
        stable_instance_key: 7 << 16,
        draw_ordinal: 0,
        phase: RenderPhase::Opaque3d,
        disabled_passes: Default::default(),
        shader_quality: Default::default(),
    };
    let second = CachedMeshDrawKey {
        stable_instance_key: (7 << 16) | 1,
        ..first
    };

    cache.store(
        first,
        &state,
        test_command(RenderPhase::Opaque3d, 1).static_payload(),
        1,
    );
    cache.store(
        second,
        &state,
        test_command(RenderPhase::Opaque3d, 2).static_payload(),
        1,
    );

    assert_eq!(cache.len(), 2);
    assert!(cache.lookup(&first, &state, 2).is_some());
    assert!(cache.lookup(&second, &state, 2).is_some());
}

fn test_command(phase: RenderPhase, sort_key: u64) -> MeshDrawCommand {
    MeshDrawCommand::new(
        phase,
        MeshPassPipelineKind::Base,
        default_pipeline_key(),
        MeshPipelineVariantId::new(1),
        sort_key,
        DrawInstanceSource::GpuSceneInstance {
            first_instance_index: 1,
            instance_count: 1,
        },
        MeshGeometryHandle::test(1),
        MeshDrawArgs::direct_indexed(0, 3).with_instance_span(1, 1),
    )
}

fn batch(phase: MeshDrawQueuePhase, mobility: Mobility, indirect: bool) -> MeshBatchRef {
    MeshBatchRef::new(
        MeshDrawQueueProfile::new(
            phase,
            MeshDrawGeometrySource::Prepared,
            mobility,
            indirect,
            false,
            false,
        ),
        default_pipeline_key(),
        RenderPhaseSortComponents::new(0.0, 1),
        MeshGeometryHandle::test(1),
        MeshDrawArgs::direct_indexed(0, 3),
    )
    .with_gpu_scene_instance_span(1, 1)
}

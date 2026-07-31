use crate::core::framework::render::{
    RenderMeshStaticState, RenderPhase, RenderPhaseSortComponents,
};
use crate::core::framework::scene::Mobility;
use crate::graphics::scene::resources::default_pipeline_key;
use crate::graphics::scene::scene_renderer::mesh::mesh_draw::{
    MeshDrawGeometrySource, MeshDrawQueuePhase, MeshDrawQueueProfile,
};
use crate::graphics::scene::scene_renderer::mesh::mesh_pass::{
    CachedMeshDrawCommands, CachedMeshDrawKey, DrawInstanceSource, MeshDrawArgs, MeshDrawCommand,
    MeshGeometryHandle, MeshPassPipelineKind, MeshPipelineVariantId,
};

use super::{PendingMeshCommandCacheExtractItem, commands_for_extract_item};

#[test]
fn pending_command_cache_extract_defers_rebuild_batch_on_full_hit() {
    let mut cache = CachedMeshDrawCommands::default();
    let item = item();
    store(&mut cache, item, RenderPhase::Prepass, 10);
    store(&mut cache, item, RenderPhase::Shadow, 20);
    store(&mut cache, item, RenderPhase::Opaque3d, 30);
    let mut rebuild_batch_requested = false;

    let extracted = commands_for_extract_item(
        item,
        None,
        |_| {
            rebuild_batch_requested = true;
            None
        },
        &mut cache,
        2,
    )
    .expect("full-hit static draw should not need rebuild input");

    assert_eq!(extracted.commands.len(), 3);
    assert!(!rebuild_batch_requested);
}

#[test]
fn pending_command_cache_extract_does_not_materialize_batch_for_material_phase_miss() {
    let mut cache = CachedMeshDrawCommands::default();
    let item = item();
    store(&mut cache, item, RenderPhase::Shadow, 20);
    let mut rebuild_batch_requested = false;

    let extracted = commands_for_extract_item(
        item,
        None,
        |_| {
            rebuild_batch_requested = true;
            None
        },
        &mut cache,
        2,
    );

    assert!(extracted.is_none());
    assert!(!rebuild_batch_requested);
}

fn item() -> PendingMeshCommandCacheExtractItem {
    PendingMeshCommandCacheExtractItem {
        entity: 7,
        draw_ordinal: 1,
        source_draw_index: 8,
        queue_profile: MeshDrawQueueProfile::new(
            MeshDrawQueuePhase::Opaque,
            MeshDrawGeometrySource::Prepared,
            Mobility::Static,
            false,
            false,
            false,
        ),
        static_state: RenderMeshStaticState::new(true, 11, 17),
        casts_shadow: true,
        disabled_passes: Default::default(),
        taa_reactive_mask_strength: 0.0,
        skinned: false,
    }
}

fn store(
    cache: &mut CachedMeshDrawCommands,
    item: PendingMeshCommandCacheExtractItem,
    phase: RenderPhase,
    sort_key: u64,
) {
    cache.store(
        CachedMeshDrawKey {
            entity: item.entity,
            draw_ordinal: item.draw_ordinal,
            phase,
            disabled_passes: item.disabled_passes,
        },
        &item.static_state,
        command(phase, sort_key),
        1,
    );
}

fn command(phase: RenderPhase, sort_key: u64) -> MeshDrawCommand {
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

use crate::core::framework::render::{
    CorePipelineKind, PrimitiveRelevance, RenderLayerSet, RenderMaterialAlphaMode,
    RenderMeshStaticState, RenderPhase, RenderPhaseSortComponents,
};
use crate::core::framework::scene::Mobility;
use crate::graphics::scene::resources::default_pipeline_key;
use crate::graphics::scene::scene_renderer::mesh::mesh_draw::{
    MeshDrawGeometrySource, MeshDrawQueuePhase, MeshDrawQueueProfile,
};
use crate::graphics::scene::scene_renderer::mesh::mesh_pass::{
    CachedMeshDrawCommands, CachedMeshDrawKey, DrawInstanceSource, MeshBatchRef, MeshDrawArgs,
    MeshDrawCommand, MeshGeometryHandle, MeshPassPipelineKind, MeshPipelineVariantId,
};

use super::super::pending_command_cache_plan::PendingMeshCommandCacheVisibility;
use super::{
    commands_for_extract_item_with_stats, PendingMeshCommandCacheExtractItem,
    PendingMeshCommandCacheExtractionStats,
};

#[test]
fn pending_command_cache_extract_second_frame_full_hit_reports_zero_rebuilds() {
    let mut cache = CachedMeshDrawCommands::default();
    let state = RenderMeshStaticState::new(true, 11, 17);
    let item = item(state, MeshDrawQueuePhase::Opaque, true);
    store(&mut cache, item, state, RenderPhase::Prepass, 10);
    store(&mut cache, item, state, RenderPhase::Shadow, 20);
    store(&mut cache, item, state, RenderPhase::Opaque3d, 30);
    let mut stats = PendingMeshCommandCacheExtractionStats::default();
    let mut rebuild_batch_requested = false;

    let extracted = commands_for_extract_item_with_stats(
        item,
        None,
        |_| {
            rebuild_batch_requested = true;
            None
        },
        &mut cache,
        2,
        Some(&mut stats),
    )
    .expect("full-hit second frame should skip MeshDraw with cached commands");

    assert_eq!(extracted.commands.len(), 3);
    assert_eq!(extracted.cache_stats.cached_command_hit_count, 3);
    assert_eq!(extracted.cache_stats.command_rebuild_count, 0);
    assert_eq!(extracted.cache_stats.cache_miss_count, 0);
    assert_eq!(extracted.cache_stats.cache_invalidated_material_count, 0);
    assert_eq!(stats, PendingMeshCommandCacheExtractionStats::default());
    assert!(!rebuild_batch_requested);
    cache.retain_generation(2);
    assert_eq!(cache.len(), 3);
}

#[test]
fn pending_command_cache_extract_rebuilds_shadow_material_invalidation_before_mesh_draw() {
    let mut cache = CachedMeshDrawCommands::default();
    let previous_state = RenderMeshStaticState::new(true, 11, 17);
    let changed_material = RenderMeshStaticState::new(true, 11, 23);
    let item = item(changed_material, MeshDrawQueuePhase::Opaque, true);
    let visibility = shadow_only_visibility();
    store(&mut cache, item, previous_state, RenderPhase::Shadow, 20);
    let batch = batch(item, Some(visibility));
    let mut stats = PendingMeshCommandCacheExtractionStats::default();

    let extracted = commands_for_extract_item_with_stats(
        item,
        Some(visibility),
        |_| Some(batch.clone()),
        &mut cache,
        2,
        Some(&mut stats),
    )
    .expect("opaque shadow can rebuild without material bind groups");

    assert_eq!(extracted.commands.len(), 1);
    assert_eq!(extracted.commands[0].phase, RenderPhase::Shadow);
    assert_eq!(extracted.cache_stats.cached_command_hit_count, 0);
    assert_eq!(extracted.cache_stats.cache_miss_count, 0);
    assert_eq!(extracted.cache_stats.command_rebuild_count, 1);
    assert_eq!(extracted.cache_stats.cache_invalidated_material_count, 1);
    assert_eq!(stats, PendingMeshCommandCacheExtractionStats::default());
    cache.retain_generation(2);
    assert_eq!(cache.len(), 1);
}

fn item(
    static_state: RenderMeshStaticState,
    phase: MeshDrawQueuePhase,
    casts_shadow: bool,
) -> PendingMeshCommandCacheExtractItem {
    PendingMeshCommandCacheExtractItem {
        entity: 7,
        draw_ordinal: 1,
        source_draw_index: 8,
        queue_profile: MeshDrawQueueProfile::new(
            phase,
            MeshDrawGeometrySource::Prepared,
            Mobility::Static,
            false,
            false,
            false,
        ),
        static_state,
        casts_shadow,
        taa_reactive_mask_strength: 0.0,
        skinned: false,
    }
}

fn store(
    cache: &mut CachedMeshDrawCommands,
    item: PendingMeshCommandCacheExtractItem,
    state: RenderMeshStaticState,
    phase: RenderPhase,
    sort_key: u64,
) {
    cache.store(
        CachedMeshDrawKey {
            entity: item.entity,
            draw_ordinal: item.draw_ordinal,
            phase,
        },
        &state,
        command(phase, sort_key),
        1,
    );
}

fn batch(
    item: PendingMeshCommandCacheExtractItem,
    visibility: Option<PendingMeshCommandCacheVisibility>,
) -> MeshBatchRef {
    let (relevance, main_view_visible, shadow_view_visible) = visibility
        .map(|visibility| {
            (
                Some(visibility.relevance),
                visibility.main_view_visible,
                visibility.shadow_view_visible,
            )
        })
        .unwrap_or((None, true, true));

    MeshBatchRef::new(
        item.queue_profile,
        default_pipeline_key(),
        RenderPhaseSortComponents::new(0.0, 10),
        MeshGeometryHandle::test(1),
        MeshDrawArgs::direct_indexed(0, 3),
    )
    .with_source_draw_index(item.source_draw_index)
    .with_cache_identity(item.entity, item.draw_ordinal)
    .with_static_state(item.static_state)
    .with_casts_shadow(item.casts_shadow)
    .with_visibility(relevance, main_view_visible, shadow_view_visible)
    .with_gpu_scene_instance_span(1, 1)
}

fn shadow_only_visibility() -> PendingMeshCommandCacheVisibility {
    PendingMeshCommandCacheVisibility {
        relevance: PrimitiveRelevance::for_mesh_view(
            &RenderLayerSet::layer(1),
            CorePipelineKind::Core3d,
            &RenderLayerSet::layer(2),
            Mobility::Static,
            RenderMaterialAlphaMode::Opaque,
        ),
        main_view_visible: false,
        shadow_view_visible: true,
    }
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

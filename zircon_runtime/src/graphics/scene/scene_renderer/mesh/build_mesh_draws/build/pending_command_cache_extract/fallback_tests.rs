use crate::core::framework::render::{
    CorePipelineKind, PrimitiveRelevance, RenderLayerSet, RenderMaterialAlphaMode,
    RenderMeshStaticState, RenderPhaseSortComponents,
};
use crate::core::framework::scene::Mobility;
use crate::graphics::scene::resources::default_pipeline_key;
use crate::graphics::scene::scene_renderer::mesh::mesh_draw::{
    MeshDrawGeometrySource, MeshDrawQueuePhase, MeshDrawQueueProfile,
};
use crate::graphics::scene::scene_renderer::mesh::mesh_pass::{
    CachedMeshDrawCommands, MeshBatchRef, MeshDrawArgs, MeshGeometryHandle,
};

use super::super::pending_command_cache_plan::PendingMeshCommandCacheVisibility;
use super::{
    commands_for_extract_item_with_stats, PendingMeshCommandCacheExtractItem,
    PendingMeshCommandCacheExtractionStats,
};

#[test]
fn pending_command_cache_extract_records_material_phase_residual_fallback() {
    let mut cache = CachedMeshDrawCommands::default();
    let item = item(MeshDrawQueuePhase::Opaque);
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
    );

    assert!(extracted.is_none());
    assert!(!rebuild_batch_requested);
    assert_eq!(stats.residual_material_phase_draw_count, 1);
}

#[test]
fn pending_command_cache_extract_records_missing_rebuild_input_fallback() {
    let mut cache = CachedMeshDrawCommands::default();
    let item = item(MeshDrawQueuePhase::Opaque);
    let mut stats = PendingMeshCommandCacheExtractionStats::default();
    let mut rebuild_batch_requested = false;

    let extracted = commands_for_extract_item_with_stats(
        item,
        Some(shadow_only_visibility()),
        |_| {
            rebuild_batch_requested = true;
            None
        },
        &mut cache,
        2,
        Some(&mut stats),
    );

    assert!(extracted.is_none());
    assert!(rebuild_batch_requested);
    assert_eq!(stats.residual_rebuild_input_missing_draw_count, 1);
}

#[test]
fn pending_command_cache_extract_records_rebuild_rejected_fallback() {
    let mut cache = CachedMeshDrawCommands::default();
    let item = item(MeshDrawQueuePhase::AlphaMask);
    let batch = batch(item, Some(shadow_only_visibility()));
    let mut stats = PendingMeshCommandCacheExtractionStats::default();

    let extracted = commands_for_extract_item_with_stats(
        item,
        Some(shadow_only_visibility()),
        |_| Some(batch.clone()),
        &mut cache,
        2,
        Some(&mut stats),
    );

    assert!(extracted.is_none());
    assert_eq!(stats.residual_rebuild_rejected_draw_count, 1);
}

fn item(phase: MeshDrawQueuePhase) -> PendingMeshCommandCacheExtractItem {
    PendingMeshCommandCacheExtractItem {
        entity: 7,
        draw_ordinal: 1,
        source_draw_index: 0,
        queue_profile: MeshDrawQueueProfile::new(
            phase,
            MeshDrawGeometrySource::Prepared,
            Mobility::Static,
            false,
            false,
            false,
        ),
        static_state: RenderMeshStaticState::new(true, 11, 17),
        casts_shadow: true,
        taa_reactive_mask_strength: 0.0,
        skinned: false,
    }
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

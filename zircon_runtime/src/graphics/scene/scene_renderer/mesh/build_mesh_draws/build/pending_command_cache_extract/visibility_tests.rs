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
    CachedMeshDrawCommands, CachedMeshDrawKey, DrawInstanceSource, MeshDrawArgs, MeshDrawCommand,
    MeshGeometryHandle, MeshPassPipelineKind, MeshPipelineVariantId,
};

use super::super::pending_command_cache_plan::PendingMeshCommandCacheVisibility;
use super::{PendingMeshCommandCacheExtractItem, commands_for_extract_item};

#[test]
fn pending_command_cache_extract_marks_visibility_pruned_static_draw() {
    let mut cache = CachedMeshDrawCommands::default();
    let item = item(6);
    let visibility = hidden_visibility();
    let mut rebuild_batch_requested = false;

    let extracted = commands_for_extract_item(
        item,
        Some(visibility),
        |_| {
            rebuild_batch_requested = true;
            None
        },
        &mut cache,
        4,
    )
    .expect("fully visibility-pruned static draw should skip MeshDraw without commands");

    assert!(extracted.commands.is_empty());
    assert_eq!(extracted.cache_stats, Default::default());
    assert!(extracted.visibility_pruned);
    assert!(!rebuild_batch_requested);
}

#[test]
fn pending_command_cache_extract_keeps_hidden_static_entries_alive_without_emitting_commands() {
    let mut cache = CachedMeshDrawCommands::default();
    let item = item(6);
    store_cacheable_commands(&mut cache, item, item.static_state);
    let mut rebuild_batch_requested = false;

    let hidden = commands_for_extract_item(
        item,
        Some(hidden_visibility()),
        |_| {
            rebuild_batch_requested = true;
            None
        },
        &mut cache,
        2,
    )
    .expect("hidden static draw should retain cache entries without materialization");

    assert!(hidden.commands.is_empty());
    assert_eq!(hidden.cache_stats, Default::default());
    assert!(hidden.visibility_pruned);
    assert!(!rebuild_batch_requested);
    cache.retain_generation(2);
    assert_eq!(cache.len(), 3);

    let visible = commands_for_extract_item(item, None, |_| None, &mut cache, 3)
        .expect("visible static draw should reuse entries retained during the hidden frame");

    assert_eq!(visible.commands.len(), 3);
    assert_eq!(visible.cache_stats.cached_command_hit_count, 3);
    assert_eq!(visible.cache_stats.command_rebuild_count, 0);
}

#[test]
fn pending_command_cache_extract_does_not_refresh_hidden_entries_after_static_revision_change() {
    let mut cache = CachedMeshDrawCommands::default();
    let item = item(6);
    store_cacheable_commands(&mut cache, item, item.static_state);
    let mut changed_item = item;
    changed_item.static_state = RenderMeshStaticState::new(true, 11, 23);

    let hidden = commands_for_extract_item(
        changed_item,
        Some(hidden_visibility()),
        |_| None,
        &mut cache,
        2,
    )
    .expect("hidden static draw should not materialize stale cached commands");

    assert!(hidden.commands.is_empty());
    assert_eq!(hidden.cache_stats, Default::default());
    cache.retain_generation(2);
    assert_eq!(cache.len(), 0);
}

#[test]
fn pending_command_cache_extract_keeps_main_view_pruned_entries_alive_when_shadow_is_visible() {
    let mut cache = CachedMeshDrawCommands::default();
    let item = item(6);
    store_cacheable_commands(&mut cache, item, item.static_state);
    let shadow_only = commands_for_extract_item(
        item,
        Some(shadow_only_visibility()),
        |_| None,
        &mut cache,
        2,
    )
    .expect("shadow-visible static draw should use only its shadow command");

    assert_eq!(shadow_only.commands.len(), 1);
    assert_eq!(shadow_only.commands[0].phase, RenderPhase::Shadow);
    assert_eq!(shadow_only.cache_stats.cached_command_hit_count, 1);
    cache.retain_generation(2);
    assert_eq!(cache.len(), 3);
}

fn item(source_draw_index: usize) -> PendingMeshCommandCacheExtractItem {
    PendingMeshCommandCacheExtractItem {
        entity: 7,
        stable_instance_key: (7 << 16) | 1,
        draw_ordinal: 1,
        source_draw_index,
        sort_components: RenderPhaseSortComponents::new(0.0, 10),
        gpu_scene_instance_span: Some((1, 1)),
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

fn hidden_visibility() -> PendingMeshCommandCacheVisibility {
    PendingMeshCommandCacheVisibility {
        relevance: PrimitiveRelevance::empty(),
        main_view_visible: false,
        shadow_view_visible: false,
    }
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

fn store_cacheable_commands(
    cache: &mut CachedMeshDrawCommands,
    item: PendingMeshCommandCacheExtractItem,
    state: RenderMeshStaticState,
) {
    for (phase, sort_key) in [
        (RenderPhase::Prepass, 10),
        (RenderPhase::Shadow, 20),
        (RenderPhase::Opaque3d, 30),
    ] {
        cache.store(
            CachedMeshDrawKey {
                stable_instance_key: item.stable_instance_key,
                draw_ordinal: item.draw_ordinal,
                phase,
                disabled_passes: item.disabled_passes,
                shader_quality: Default::default(),
            },
            &state,
            command(phase, sort_key).static_payload(),
            1,
        );
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

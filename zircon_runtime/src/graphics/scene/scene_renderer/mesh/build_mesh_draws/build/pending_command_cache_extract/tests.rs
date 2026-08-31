use crate::core::framework::render::{
    CorePipelineKind, PrimitiveRelevance, RenderLayerSet, RenderMaterialAlphaMode,
    RenderMeshStaticState, RenderPhase, RenderPhaseSortComponents, packed_sort_key_u64,
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
    PendingMeshCommandCacheExtractItem, cached_commands_for_extract_item, commands_for_extract_item,
};

#[test]
fn pending_command_cache_extracts_full_hit_without_rebuild_input() {
    let mut cache = CachedMeshDrawCommands::default();
    let item = item(MeshDrawQueuePhase::Opaque, true, 9);
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
            &item.static_state,
            command(phase, sort_key).static_payload(),
            1,
        );
    }

    let commands = cached_commands_for_extract_item(item, None, &mut cache, 2)
        .expect("all cacheable phases should hit");

    assert_eq!(commands.len(), 3);
    assert!(commands.iter().all(|command| command.source_entity == 7));
    assert!(
        commands
            .iter()
            .all(|command| command.source_draw_index == 9)
    );
    for command in &commands {
        assert_eq!(
            command.sort_key,
            packed_sort_key_u64(
                command.phase,
                item.sort_components,
                command.pipeline_variant_id.value(),
                0,
            )
        );
        assert_instance_span(command, 53, 7);
    }
    cache.retain_generation(2);
    assert_eq!(cache.len(), 3);
}

#[test]
fn pending_command_cache_extract_rejects_cache_hit_without_current_gpu_scene_span() {
    let mut cache = CachedMeshDrawCommands::default();
    let mut item = item(MeshDrawQueuePhase::Opaque, true, 9);
    cache.store(
        CachedMeshDrawKey {
            stable_instance_key: item.stable_instance_key,
            draw_ordinal: item.draw_ordinal,
            phase: RenderPhase::Prepass,
            disabled_passes: item.disabled_passes,
            shader_quality: Default::default(),
        },
        &item.static_state,
        command(RenderPhase::Prepass, 10).static_payload(),
        1,
    );
    item.gpu_scene_instance_span = None;

    assert!(cached_commands_for_extract_item(item, None, &mut cache, 2).is_none());
}

#[test]
fn pending_command_cache_extract_waits_for_residual_path_on_partial_miss() {
    let mut cache = CachedMeshDrawCommands::default();
    let item = item(MeshDrawQueuePhase::Opaque, true, 0);
    cache.store(
        CachedMeshDrawKey {
            stable_instance_key: item.stable_instance_key,
            draw_ordinal: item.draw_ordinal,
            phase: RenderPhase::Prepass,
            disabled_passes: Default::default(),
            shader_quality: Default::default(),
        },
        &item.static_state,
        command(RenderPhase::Prepass, 10).static_payload(),
        1,
    );

    assert!(cached_commands_for_extract_item(item, None, &mut cache, 2).is_none());
}

#[test]
fn pending_command_cache_extract_rejects_reactive_or_skinned_draws() {
    let mut cache = CachedMeshDrawCommands::default();
    let mut reactive = item(MeshDrawQueuePhase::Opaque, true, 0);
    reactive.taa_reactive_mask_strength = 0.5;
    let mut skinned = item(MeshDrawQueuePhase::Opaque, true, 0);
    skinned.skinned = true;

    assert!(cached_commands_for_extract_item(reactive, None, &mut cache, 2).is_none());
    assert!(cached_commands_for_extract_item(skinned, None, &mut cache, 2).is_none());
}

#[test]
fn pending_command_cache_extract_rebuilds_shadow_only_miss_before_mesh_draw() {
    let mut cache = CachedMeshDrawCommands::default();
    let item = item(MeshDrawQueuePhase::Opaque, true, 4);
    let visibility = shadow_only_visibility();
    let batch = batch(item, Some(visibility));

    let extracted = commands_for_extract_item(
        item,
        Some(visibility),
        |_| Some(batch.clone()),
        &mut cache,
        3,
    )
    .expect("shadow-only static draw should rebuild before MeshDraw");

    assert_eq!(extracted.commands.len(), 1);
    assert_eq!(extracted.commands[0].phase, RenderPhase::Shadow);
    assert_eq!(
        extracted.commands[0].pipeline_kind,
        MeshPassPipelineKind::ShadowDepth
    );
    assert_ne!(
        extracted.commands[0].pipeline_variant_id,
        MeshPipelineVariantId::new(0)
    );
    assert_eq!(extracted.cache_stats.cache_miss_count, 1);
    assert_eq!(extracted.cache_stats.command_rebuild_count, 1);
    assert_eq!(cache.len(), 1);
}

#[test]
fn pending_command_cache_extract_rebuilds_non_material_miss_when_material_phase_hits() {
    let mut cache = CachedMeshDrawCommands::default();
    let item = item(MeshDrawQueuePhase::Opaque, true, 2);
    for (phase, sort_key) in [(RenderPhase::Prepass, 10), (RenderPhase::Opaque3d, 30)] {
        cache.store(
            CachedMeshDrawKey {
                stable_instance_key: item.stable_instance_key,
                draw_ordinal: item.draw_ordinal,
                phase,
                disabled_passes: item.disabled_passes,
                shader_quality: Default::default(),
            },
            &item.static_state,
            command(phase, sort_key).static_payload(),
            1,
        );
    }
    let batch = batch(item, None);

    let extracted = commands_for_extract_item(item, None, |_| Some(batch.clone()), &mut cache, 5)
        .expect("opaque shadow miss can rebuild when material-bound phases hit");

    assert_eq!(extracted.commands.len(), 3);
    assert_eq!(extracted.cache_stats.cached_command_hit_count, 2);
    assert_eq!(extracted.cache_stats.cache_miss_count, 1);
    assert_eq!(extracted.cache_stats.command_rebuild_count, 1);
    cache.retain_generation(5);
    assert_eq!(cache.len(), 3);
}

fn item(
    phase: MeshDrawQueuePhase,
    casts_shadow: bool,
    source_draw_index: usize,
) -> PendingMeshCommandCacheExtractItem {
    PendingMeshCommandCacheExtractItem {
        entity: 7,
        stable_instance_key: (7 << 16) | 1,
        draw_ordinal: 1,
        source_draw_index,
        sort_components: RenderPhaseSortComponents::new(0.75, 61),
        gpu_scene_instance_span: Some((53, 7)),
        queue_profile: MeshDrawQueueProfile::new(
            phase,
            MeshDrawGeometrySource::Prepared,
            Mobility::Static,
            false,
            false,
            false,
        ),
        static_state: RenderMeshStaticState::new(true, 11, 17),
        casts_shadow,
        disabled_passes: Default::default(),
        taa_reactive_mask_strength: 0.0,
        skinned: false,
    }
}

fn assert_instance_span(command: &MeshDrawCommand, expected_first: u32, expected_count: u32) {
    match &command.instance_source {
        DrawInstanceSource::GpuSceneInstance {
            first_instance_index,
            instance_count,
        } => {
            assert_eq!(*first_instance_index, expected_first);
            assert_eq!(*instance_count, expected_count);
        }
    }
    match &command.draw_args {
        MeshDrawArgs::DirectIndexed {
            first_instance,
            instance_count,
            ..
        } => {
            assert_eq!(*first_instance, expected_first);
            assert_eq!(*instance_count, expected_count);
        }
        MeshDrawArgs::IndexedIndirect { .. } => panic!("test command must be direct"),
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
    .with_source_draw_index(item.source_draw_index)
    .with_cache_identity(item.entity, item.stable_instance_key, item.draw_ordinal)
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

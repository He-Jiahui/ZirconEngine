use crate::core::framework::render::{PrimitiveRelevance, RenderMeshStaticState};
use crate::core::framework::scene::Mobility;
use crate::graphics::scene::scene_renderer::mesh::mesh_draw::{
    MeshDrawGeometrySource, MeshDrawQueuePhase, MeshDrawQueueProfile,
};
use crate::graphics::scene::scene_renderer::mesh::mesh_pass::CachedMeshDrawCommands;

use super::super::pending_command_cache_plan::PendingMeshCommandCacheVisibility;
use super::{commands_for_extract_item, PendingMeshCommandCacheExtractItem};

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

fn item(source_draw_index: usize) -> PendingMeshCommandCacheExtractItem {
    PendingMeshCommandCacheExtractItem {
        entity: 7,
        stable_instance_key: (7 << 16) | 1,
        draw_ordinal: 1,
        source_draw_index,
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

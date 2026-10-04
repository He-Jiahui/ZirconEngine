use crate::core::framework::render::{
    CorePipelineKind, PrimitiveRelevance, RenderLayerSet, RenderMaterialAlphaMode,
    RenderMeshStaticState,
};
use crate::core::framework::scene::Mobility;
use crate::graphics::scene::scene_renderer::mesh::mesh_draw::{
    MeshDrawGeometrySource, MeshDrawQueuePhase, MeshDrawQueueProfile,
};

use super::{
    summarize_pending_mesh_command_cache_plan_items, PendingMeshCommandCachePlanItem,
    PendingMeshCommandCacheVisibility,
};

#[test]
fn pending_command_cache_plan_counts_static_opaque_phase_candidates() {
    let stats = summarize_pending_mesh_command_cache_plan_items([(
        item(MeshDrawQueuePhase::Opaque, Mobility::Static, true),
        None,
    )]);

    assert_eq!(stats.static_command_cache_draw_candidate_count, 1);
    assert_eq!(stats.static_command_cache_phase_candidate_count, 3);
    assert_eq!(stats.static_command_cache_depth_prepass_candidate_count, 1);
    assert_eq!(stats.static_command_cache_shadow_candidate_count, 1);
    assert_eq!(stats.static_command_cache_opaque_candidate_count, 1);
    assert_eq!(stats.static_command_cache_alpha_mask_candidate_count, 0);
}

#[test]
fn pending_command_cache_plan_counts_alpha_mask_without_shadow_material() {
    let stats = summarize_pending_mesh_command_cache_plan_items([(
        item(MeshDrawQueuePhase::AlphaMask, Mobility::Static, false),
        None,
    )]);

    assert_eq!(stats.static_command_cache_draw_candidate_count, 1);
    assert_eq!(stats.static_command_cache_phase_candidate_count, 2);
    assert_eq!(stats.static_command_cache_depth_prepass_candidate_count, 1);
    assert_eq!(stats.static_command_cache_shadow_candidate_count, 0);
    assert_eq!(stats.static_command_cache_alpha_mask_candidate_count, 1);
}

#[test]
fn pending_command_cache_plan_rejects_dynamic_transparent_and_missing_revisions() {
    let missing_revisions = PendingMeshCommandCachePlanItem::new(
        profile(MeshDrawQueuePhase::Opaque, Mobility::Static, false),
        RenderMeshStaticState::default(),
        true,
    );
    let stats = summarize_pending_mesh_command_cache_plan_items([
        (
            item(MeshDrawQueuePhase::Opaque, Mobility::Dynamic, true),
            None,
        ),
        (
            item(MeshDrawQueuePhase::Transparent, Mobility::Static, true),
            None,
        ),
        (missing_revisions, None),
    ]);

    assert_eq!(stats.static_command_cache_draw_candidate_count, 0);
    assert_eq!(stats.static_command_cache_phase_candidate_count, 0);
}

#[test]
fn pending_command_cache_plan_keeps_identity_candidate_when_visibility_prunes_phases() {
    let hidden_main_and_shadow =
        PendingMeshCommandCacheVisibility::new(PrimitiveRelevance::empty(), false, false);
    let stats = summarize_pending_mesh_command_cache_plan_items([(
        item(MeshDrawQueuePhase::Opaque, Mobility::Static, true),
        Some(hidden_main_and_shadow),
    )]);

    assert_eq!(stats.static_command_cache_draw_candidate_count, 1);
    assert_eq!(stats.static_command_cache_phase_candidate_count, 0);
}

#[test]
fn pending_command_cache_plan_keeps_shadow_candidate_for_hidden_main_view() {
    let hidden_main_shadow_visible = PendingMeshCommandCacheVisibility::new(
        relevance(RenderMaterialAlphaMode::Mask { cutoff: 0.5 }, false),
        false,
        true,
    );
    let stats = summarize_pending_mesh_command_cache_plan_items([(
        item(MeshDrawQueuePhase::AlphaMask, Mobility::Static, true),
        Some(hidden_main_shadow_visible),
    )]);

    assert_eq!(stats.static_command_cache_draw_candidate_count, 1);
    assert_eq!(stats.static_command_cache_phase_candidate_count, 1);
    assert_eq!(stats.static_command_cache_shadow_candidate_count, 1);
    assert_eq!(stats.static_command_cache_alpha_mask_candidate_count, 0);
}

fn item(
    phase: MeshDrawQueuePhase,
    mobility: Mobility,
    casts_shadow: bool,
) -> PendingMeshCommandCachePlanItem {
    PendingMeshCommandCachePlanItem::new(
        profile(phase, mobility, false),
        RenderMeshStaticState::new(true, 11, 17),
        casts_shadow,
    )
}

fn profile(
    phase: MeshDrawQueuePhase,
    mobility: Mobility,
    uses_indirect_draw: bool,
) -> MeshDrawQueueProfile {
    MeshDrawQueueProfile::new(
        phase,
        MeshDrawGeometrySource::Prepared,
        mobility,
        uses_indirect_draw,
        false,
        false,
    )
}

fn relevance(
    alpha_mode: RenderMaterialAlphaMode,
    render_layer_visible: bool,
) -> PrimitiveRelevance {
    let camera_layers = RenderLayerSet::layer(0);
    let render_layers = if render_layer_visible {
        RenderLayerSet::layer(0)
    } else {
        RenderLayerSet::layer(1)
    };
    PrimitiveRelevance::for_mesh_view(
        &camera_layers,
        CorePipelineKind::Core3d,
        &render_layers,
        Mobility::Static,
        alpha_mode,
    )
}

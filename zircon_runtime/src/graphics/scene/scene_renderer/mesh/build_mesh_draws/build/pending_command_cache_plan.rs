use crate::core::framework::render::{PrimitiveRelevance, RenderMeshStaticState, RenderPhase};
use crate::graphics::scene::scene_renderer::mesh::mesh_draw::{
    MeshDrawQueuePhase, MeshDrawQueueProfile,
};

use super::geometry_source_selection::{
    pending_draw_has_enabled_skinned_gpu_source, pending_mesh_draw_queue_profile,
};
use super::pending_mesh_draw::PendingMeshDraw;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct PendingMeshCommandCachePlanStats {
    pub(crate) static_command_cache_draw_candidate_count: usize,
    pub(crate) static_command_cache_phase_candidate_count: usize,
    pub(crate) static_command_cache_depth_prepass_candidate_count: usize,
    pub(crate) static_command_cache_shadow_candidate_count: usize,
    pub(crate) static_command_cache_opaque_candidate_count: usize,
    pub(crate) static_command_cache_alpha_mask_candidate_count: usize,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct PendingMeshCommandCacheVisibility {
    pub(super) relevance: PrimitiveRelevance,
    pub(super) main_view_visible: bool,
    pub(super) shadow_view_visible: bool,
}

#[derive(Clone, Copy, Debug)]
struct PendingMeshCommandCachePlanItem {
    queue_profile: MeshDrawQueueProfile,
    static_state: RenderMeshStaticState,
    casts_shadow: bool,
}

impl PendingMeshCommandCacheVisibility {
    pub(super) const fn new(
        relevance: PrimitiveRelevance,
        main_view_visible: bool,
        shadow_view_visible: bool,
    ) -> Self {
        Self {
            relevance,
            main_view_visible,
            shadow_view_visible,
        }
    }
}

impl PendingMeshCommandCachePlanItem {
    const fn new(
        queue_profile: MeshDrawQueueProfile,
        static_state: RenderMeshStaticState,
        casts_shadow: bool,
    ) -> Self {
        Self {
            queue_profile,
            static_state,
            casts_shadow,
        }
    }
}

pub(super) fn summarize_pending_mesh_command_cache_plan(
    pending_draws: &[PendingMeshDraw],
    visibility_for_instance: impl Fn(u64) -> Option<PendingMeshCommandCacheVisibility>,
) -> PendingMeshCommandCachePlanStats {
    summarize_pending_mesh_command_cache_plan_items(pending_draws.iter().map(|pending_draw| {
        (
            pending_mesh_command_cache_plan_item(pending_draw),
            visibility_for_instance(pending_draw.stable_instance_key),
        )
    }))
}

fn summarize_pending_mesh_command_cache_plan_items(
    items: impl IntoIterator<
        Item = (
            PendingMeshCommandCachePlanItem,
            Option<PendingMeshCommandCacheVisibility>,
        ),
    >,
) -> PendingMeshCommandCachePlanStats {
    let mut stats = PendingMeshCommandCachePlanStats::default();
    for (item, visibility) in items {
        if !item.static_state.has_authoritative_revisions()
            || !item.queue_profile.static_batch_eligible()
        {
            continue;
        }

        stats.static_command_cache_draw_candidate_count += 1;
        accumulate_cacheable_phases(&mut stats, item, visibility);
    }
    stats
}

fn pending_mesh_command_cache_plan_item(
    pending_draw: &PendingMeshDraw,
) -> PendingMeshCommandCachePlanItem {
    PendingMeshCommandCachePlanItem::new(
        pending_mesh_draw_queue_profile(
            pending_draw,
            pending_draw_has_enabled_skinned_gpu_source(pending_draw),
        ),
        if pending_draw.material.uniform_override_payload.is_some() {
            RenderMeshStaticState::from_transform_static(false)
        } else {
            pending_draw.static_state
        },
        pending_draw.material.common.cast_shadows.casts_shadows(),
    )
}

fn accumulate_cacheable_phases(
    stats: &mut PendingMeshCommandCachePlanStats,
    item: PendingMeshCommandCachePlanItem,
    visibility: Option<PendingMeshCommandCacheVisibility>,
) {
    if item.queue_profile.early_z_eligible()
        && relevant_to_main_phase(visibility, RenderPhase::Prepass)
    {
        stats.static_command_cache_phase_candidate_count += 1;
        stats.static_command_cache_depth_prepass_candidate_count += 1;
    }

    if item.casts_shadow && relevant_to_shadow_view(visibility, item.casts_shadow) {
        stats.static_command_cache_phase_candidate_count += 1;
        stats.static_command_cache_shadow_candidate_count += 1;
    }

    match item.queue_profile.phase() {
        MeshDrawQueuePhase::Opaque if relevant_to_main_phase(visibility, RenderPhase::Opaque3d) => {
            stats.static_command_cache_phase_candidate_count += 1;
            stats.static_command_cache_opaque_candidate_count += 1;
        }
        MeshDrawQueuePhase::AlphaMask
            if relevant_to_main_phase(visibility, RenderPhase::AlphaMask3d) =>
        {
            stats.static_command_cache_phase_candidate_count += 1;
            stats.static_command_cache_alpha_mask_candidate_count += 1;
        }
        MeshDrawQueuePhase::Transparent
        | MeshDrawQueuePhase::Opaque
        | MeshDrawQueuePhase::AlphaMask => {}
    }
}

fn relevant_to_main_phase(
    visibility: Option<PendingMeshCommandCacheVisibility>,
    phase: RenderPhase,
) -> bool {
    visibility
        .map(|visibility| {
            visibility.main_view_visible && visibility.relevance.is_relevant_to_phase(phase)
        })
        .unwrap_or(true)
}

fn relevant_to_shadow_view(
    visibility: Option<PendingMeshCommandCacheVisibility>,
    casts_shadow: bool,
) -> bool {
    visibility
        .map(|visibility| visibility.shadow_view_visible && visibility.relevance.shadow_caster())
        .unwrap_or(casts_shadow)
}

#[cfg(test)]
mod tests {
    use crate::core::framework::render::{
        CorePipelineKind, PrimitiveRelevance, RenderLayerSet, RenderMaterialAlphaMode,
        RenderMeshStaticState,
    };
    use crate::core::framework::scene::Mobility;
    use crate::graphics::scene::scene_renderer::mesh::mesh_draw::{
        MeshDrawGeometrySource, MeshDrawQueuePhase, MeshDrawQueueProfile,
    };

    use super::{
        PendingMeshCommandCachePlanItem, PendingMeshCommandCacheVisibility,
        summarize_pending_mesh_command_cache_plan_items,
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
}

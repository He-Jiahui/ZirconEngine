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

        // 这是候选 draw 计数；后续 phase 可再按可见性裁剪，不能当作最终发出的命令数。
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
#[path = "tests/pending_command_cache_plan.rs"]
mod tests;

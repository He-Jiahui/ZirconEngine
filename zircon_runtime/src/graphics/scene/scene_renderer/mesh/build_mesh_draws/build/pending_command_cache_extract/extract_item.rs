use crate::core::framework::render::{
    RenderMeshStaticState, RenderPhase, RenderPhaseSortComponents,
};
use crate::core::framework::scene::EntityId;
use crate::graphics::scene::resources::MaterialDisabledPasses;
use crate::graphics::scene::scene_renderer::mesh::mesh_draw::{
    MeshDrawQueuePhase, MeshDrawQueueProfile,
};

use super::super::pending_command_cache_plan::PendingMeshCommandCacheVisibility;
use super::super::{
    geometry_source_selection::pending_draw_has_enabled_skinned_gpu_source,
    pending_mesh_draw::PendingMeshDraw,
};

#[derive(Clone, Copy)]
pub(super) struct PendingMeshCommandCacheExtractItem {
    pub(super) entity: EntityId,
    pub(super) stable_instance_key: u64,
    pub(super) draw_ordinal: u32,
    pub(super) source_draw_index: usize,
    pub(super) sort_components: RenderPhaseSortComponents,
    pub(super) gpu_scene_instance_span: Option<(u32, u32)>,
    pub(super) queue_profile: MeshDrawQueueProfile,
    pub(super) static_state: RenderMeshStaticState,
    pub(super) casts_shadow: bool,
    pub(super) disabled_passes: MaterialDisabledPasses,
    pub(super) taa_reactive_mask_strength: f32,
    pub(super) skinned: bool,
}

pub(super) fn pending_mesh_command_cache_extract_item(
    pending_draw: &PendingMeshDraw,
    source_draw_index: usize,
) -> PendingMeshCommandCacheExtractItem {
    PendingMeshCommandCacheExtractItem {
        entity: pending_draw.source_entity,
        stable_instance_key: pending_draw.stable_instance_key,
        draw_ordinal: pending_draw.source_draw_ordinal,
        source_draw_index,
        sort_components: pending_draw.command_sort_input.components(),
        gpu_scene_instance_span: None,
        queue_profile: pending_mesh_draw_queue_profile(pending_draw),
        static_state: if pending_draw.material.uniform_override_payload.is_some() {
            RenderMeshStaticState::from_transform_static(false)
        } else {
            pending_draw.static_state
        },
        casts_shadow: pending_draw.material.common.cast_shadows.casts_shadows(),
        disabled_passes: pending_draw.material.disabled_passes,
        taa_reactive_mask_strength: pending_draw.material.taa_reactive_mask_strength,
        skinned: pending_draw.skinned,
    }
}

pub(super) fn can_skip_pending_mesh_draw_for_cached_commands(
    item: PendingMeshCommandCacheExtractItem,
) -> bool {
    item.static_state.has_authoritative_revisions()
        && item.queue_profile.static_batch_eligible()
        && item.taa_reactive_mask_strength <= f32::EPSILON
        && !item.skinned
}

pub(super) fn cacheable_phases_for_extract_item(
    item: PendingMeshCommandCacheExtractItem,
    visibility: Option<PendingMeshCommandCacheVisibility>,
) -> [Option<RenderPhase>; 3] {
    cacheable_phase_slots_for_extract_item(item)
        .map(|phase| phase.filter(|phase| phase_is_visible(visibility, *phase, item.casts_shadow)))
}

pub(super) fn cacheable_phase_slots_for_extract_item(
    item: PendingMeshCommandCacheExtractItem,
) -> [Option<RenderPhase>; 3] {
    let depth_prepass = (!item.disabled_passes.disables_depth_prepass()
        && item.queue_profile.early_z_eligible())
    .then_some(RenderPhase::Prepass);
    let shadow = (!item.disabled_passes.disables_shadow() && item.casts_shadow)
        .then_some(RenderPhase::Shadow);
    let base = (!item.disabled_passes.disables_base())
        .then(|| match item.queue_profile.phase() {
            MeshDrawQueuePhase::Opaque => Some(RenderPhase::Opaque3d),
            MeshDrawQueuePhase::AlphaMask => Some(RenderPhase::AlphaMask3d),
            MeshDrawQueuePhase::Transparent => None,
        })
        .flatten();
    [depth_prepass, shadow, base]
}

fn pending_mesh_draw_queue_profile(pending_draw: &PendingMeshDraw) -> MeshDrawQueueProfile {
    super::super::geometry_source_selection::pending_mesh_draw_queue_profile(
        pending_draw,
        pending_draw_has_enabled_skinned_gpu_source(pending_draw),
    )
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

fn phase_is_visible(
    visibility: Option<PendingMeshCommandCacheVisibility>,
    phase: RenderPhase,
    casts_shadow: bool,
) -> bool {
    match phase {
        RenderPhase::Shadow => relevant_to_shadow_view(visibility, casts_shadow),
        RenderPhase::Prepass | RenderPhase::Opaque3d | RenderPhase::AlphaMask3d => {
            relevant_to_main_phase(visibility, phase)
        }
        RenderPhase::Transparent3d | RenderPhase::PostProcess => false,
        _ => false,
    }
}

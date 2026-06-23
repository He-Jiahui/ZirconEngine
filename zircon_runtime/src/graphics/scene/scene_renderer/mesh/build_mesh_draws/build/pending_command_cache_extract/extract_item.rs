use crate::core::framework::render::{RenderMeshStaticState, RenderPhase};
use crate::core::framework::scene::EntityId;
use crate::graphics::scene::scene_renderer::mesh::mesh_draw::{
    MeshDrawGeometrySource, MeshDrawQueuePhase, MeshDrawQueueProfile,
};

use super::super::pending_command_cache_plan::PendingMeshCommandCacheVisibility;
use super::super::pending_mesh_draw::{PendingMeshDraw, PendingMeshGeometry};

#[derive(Clone, Copy)]
pub(super) struct PendingMeshCommandCacheExtractItem {
    pub(super) entity: EntityId,
    pub(super) draw_ordinal: u32,
    pub(super) source_draw_index: usize,
    pub(super) queue_profile: MeshDrawQueueProfile,
    pub(super) static_state: RenderMeshStaticState,
    pub(super) casts_shadow: bool,
    pub(super) taa_reactive_mask_strength: f32,
    pub(super) skinned: bool,
}

pub(super) fn pending_mesh_command_cache_extract_item(
    pending_draw: &PendingMeshDraw,
    source_draw_index: usize,
) -> PendingMeshCommandCacheExtractItem {
    PendingMeshCommandCacheExtractItem {
        entity: pending_draw.source_entity,
        draw_ordinal: pending_draw.source_draw_ordinal,
        source_draw_index,
        queue_profile: pending_mesh_draw_queue_profile(pending_draw),
        static_state: pending_draw.static_state,
        casts_shadow: pending_draw.cast_shadows,
        taa_reactive_mask_strength: pending_draw.taa_reactive_mask_strength,
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
) -> Vec<RenderPhase> {
    let mut phases = Vec::with_capacity(3);
    if item.queue_profile.early_z_eligible()
        && relevant_to_main_phase(visibility, RenderPhase::Prepass)
    {
        phases.push(RenderPhase::Prepass);
    }
    if item.casts_shadow && relevant_to_shadow_view(visibility, item.casts_shadow) {
        phases.push(RenderPhase::Shadow);
    }
    match item.queue_profile.phase() {
        MeshDrawQueuePhase::Opaque if relevant_to_main_phase(visibility, RenderPhase::Opaque3d) => {
            phases.push(RenderPhase::Opaque3d);
        }
        MeshDrawQueuePhase::AlphaMask
            if relevant_to_main_phase(visibility, RenderPhase::AlphaMask3d) =>
        {
            phases.push(RenderPhase::AlphaMask3d);
        }
        MeshDrawQueuePhase::Transparent
        | MeshDrawQueuePhase::Opaque
        | MeshDrawQueuePhase::AlphaMask => {}
    }
    phases
}

fn pending_mesh_draw_queue_profile(pending_draw: &PendingMeshDraw) -> MeshDrawQueueProfile {
    MeshDrawQueueProfile::new(
        MeshDrawQueuePhase::from_pipeline_flags(
            pending_draw.pipeline_key.is_transparent(),
            pending_draw.pipeline_key.is_alpha_mask(),
        ),
        match &pending_draw.mesh {
            PendingMeshGeometry::Prepared(_) => MeshDrawGeometrySource::Prepared,
            PendingMeshGeometry::Dynamic(_) => MeshDrawGeometrySource::Dynamic,
        },
        pending_draw.mobility,
        pending_draw.indirect_draw_ref.is_some(),
        pending_draw.skinned_gpu_source.is_some(),
        pending_draw.mesh_lod.is_some(),
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

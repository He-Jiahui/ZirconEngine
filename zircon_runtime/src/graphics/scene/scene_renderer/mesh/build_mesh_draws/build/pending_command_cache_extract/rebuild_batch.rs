use std::sync::Arc;

use crate::core::framework::render::RenderPhase;
use crate::core::framework::scene::EntityId;
use crate::graphics::scene::scene_renderer::mesh::mesh_pass::{
    MeshBatchRef, MeshDrawArgs, MeshGeometryHandle,
};

use super::super::pending_command_cache_plan::PendingMeshCommandCacheVisibility;
use super::super::pending_mesh_draw::{PendingMeshDraw, PendingMeshGeometry};
use super::extract_item::PendingMeshCommandCacheExtractItem;
use super::non_material_rebuild;

pub(super) fn pending_mesh_command_cache_rebuild_batch_for_phase(
    pending_draw: &PendingMeshDraw,
    item: PendingMeshCommandCacheExtractItem,
    visibility: Option<PendingMeshCommandCacheVisibility>,
    phase: RenderPhase,
    gpu_scene_instance_span_for_draw: &impl Fn(EntityId, u32) -> Option<(u32, u32)>,
) -> Option<MeshBatchRef> {
    if !non_material_rebuild::can_rebuild_non_material_command_phase(phase) {
        return None;
    }
    gpu_scene_instance_span_for_draw(item.entity, item.draw_ordinal).and_then(
        |(first_instance_index, instance_count)| {
            pending_mesh_command_cache_rebuild_batch(
                pending_draw,
                item,
                visibility,
                first_instance_index,
                instance_count,
            )
        },
    )
}

fn pending_mesh_command_cache_rebuild_batch(
    pending_draw: &PendingMeshDraw,
    item: PendingMeshCommandCacheExtractItem,
    visibility: Option<PendingMeshCommandCacheVisibility>,
    first_instance_index: u32,
    instance_count: u32,
) -> Option<MeshBatchRef> {
    let PendingMeshGeometry::Prepared(mesh) = &pending_draw.mesh else {
        return None;
    };
    let (primitive_relevance, main_view_visible, shadow_view_visible) = visibility
        .map(|visibility| {
            (
                Some(visibility.relevance),
                visibility.main_view_visible,
                visibility.shadow_view_visible,
            )
        })
        .unwrap_or((None, true, true));

    Some(
        MeshBatchRef::new(
            item.queue_profile,
            pending_draw.pipeline_key.clone(),
            pending_draw.command_sort_input.components(),
            MeshGeometryHandle::new(arc_id(mesh), mesh.clone()),
            MeshDrawArgs::direct_indexed(pending_draw.first_index, pending_draw.draw_index_count),
        )
        .with_source_draw_index(item.source_draw_index)
        .with_cache_identity(item.entity, item.draw_ordinal)
        .with_static_state(item.static_state)
        .with_casts_shadow(item.casts_shadow)
        .with_taa_reactive_mask_strength(item.taa_reactive_mask_strength)
        .with_visibility(primitive_relevance, main_view_visible, shadow_view_visible)
        .with_gpu_scene_instance_span(first_instance_index, instance_count),
    )
}

fn arc_id<T>(value: &Arc<T>) -> u64 {
    Arc::as_ptr(value) as usize as u64
}

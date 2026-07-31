use std::collections::{HashMap, HashSet};

use crate::core::framework::render::{
    LightmapConsumeContract, RendererCommon, render_mesh_stable_instance_key,
};
use crate::core::framework::scene::Mobility;
use crate::core::math::RenderVec4;
use crate::graphics::scene::gpu_scene::{
    GPU_PRIMITIVE_FLAG_CAST_SHADOWS, GPU_PRIMITIVE_FLAG_HAS_PREVIOUS_TRANSFORM,
    GPU_PRIMITIVE_FLAG_VISIBLE, GPU_SCENE_INVALID_PAYLOAD_SLOT, GpuInstanceData, GpuPrimitiveData,
    GpuScene, GpuSceneEntry, GpuSceneUploadReport,
};
use crate::graphics::scene::resources::GpuMeshResource;

use super::super::super::super::primitives::render_vec4_or;
use super::pending_mesh_draw::{PendingMeshDraw, PendingSkinnedGpuSource};
use super::previous_skinned_palette::{
    previous_skinned_gpu_state_for_gpu_scene_entry, skinned_gpu_source_state_for_pending_draw,
    skinned_joint_palette_state_for_pending_draw,
};

#[derive(Clone, Copy)]
pub(super) struct SyncedGpuSceneEntry {
    pub(super) entry: GpuSceneEntry,
    pub(super) has_previous_velocity_transform: bool,
}

pub(super) fn sync_gpu_scene_pending_draws(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    gpu_scene: &mut GpuScene,
    pending_draws: &mut [PendingMeshDraw],
    lightmaps: Option<&LightmapConsumeContract>,
) -> (GpuSceneUploadReport, HashMap<u64, SyncedGpuSceneEntry>) {
    let mut live_keys = HashSet::new();
    let mut entries = HashMap::new();
    for pending_draw in pending_draws {
        let stable_instance_key = render_mesh_stable_instance_key(
            pending_draw.source_entity,
            pending_draw.source_draw_ordinal,
        );
        live_keys.insert(stable_instance_key);
        let entry = gpu_scene.register(device, stable_instance_key, 1);
        pending_draw.resolved_skinned_gpu_source =
            resolved_skinned_gpu_source_for_pending_draw(device, pending_draw);
        let previous_skinned_gpu_state = previous_skinned_gpu_state_for_gpu_scene_entry(
            gpu_scene,
            stable_instance_key,
            pending_draw,
        );
        pending_draw.previous_skinned_joint_palette = previous_skinned_gpu_state.joint_palette;
        pending_draw.previous_skinned_gpu_source = previous_skinned_gpu_state.source;
        gpu_scene.stage_current_skinned_joint_palette(
            stable_instance_key,
            skinned_joint_palette_state_for_pending_draw(pending_draw),
        );
        gpu_scene.stage_current_skinned_gpu_source(
            stable_instance_key,
            skinned_gpu_source_state_for_pending_draw(pending_draw),
        );
        gpu_scene.stage_current_morph_weights(
            stable_instance_key,
            pending_draw
                .source_morph_weights
                .as_ref()
                .map(Vec::as_slice),
        );
        let (previous_model_matrix, has_previous_transform) =
            previous_model_matrix_for_gpu_scene_entry(gpu_scene, pending_draw, entry);
        let has_previous_velocity_transform =
            velocity_history_is_available(pending_draw.skinned, has_previous_transform);
        gpu_scene.write_primitive(
            entry,
            primitive_data_for_pending_draw(pending_draw, entry, has_previous_velocity_transform),
        );
        gpu_scene.write_instances(
            entry,
            &[instance_data_for_pending_draw(
                pending_draw,
                entry,
                previous_model_matrix,
                stable_instance_key,
                lightmaps,
            )],
        );
        gpu_scene.set_transform_revision(stable_instance_key, pending_draw.transform_revision);
        entries.insert(
            stable_instance_key,
            SyncedGpuSceneEntry {
                entry,
                has_previous_velocity_transform,
            },
        );
    }
    gpu_scene.retain_registered_keys(&live_keys);
    (gpu_scene.flush_updates(queue), entries)
}

fn primitive_data_for_pending_draw(
    pending_draw: &PendingMeshDraw,
    entry: GpuSceneEntry,
    has_previous_velocity_transform: bool,
) -> GpuPrimitiveData {
    let flags = primitive_flags_for_renderer(
        pending_draw.common.as_ref(),
        has_previous_velocity_transform,
    );
    let payload_slot = virtual_geometry_payload_slot_for_pending_draw(pending_draw);

    GpuPrimitiveData {
        bounds_center: [
            pending_draw.model_matrix[3][0],
            pending_draw.model_matrix[3][1],
            pending_draw.model_matrix[3][2],
        ],
        bounds_radius: approximate_transform_radius(&pending_draw.model_matrix),
        tint: render_vec4_or(pending_draw.draw_tint, RenderVec4::ONE).to_array(),
        shadow_params: shadow_params_from_pending_draw(pending_draw),
        motion_params: motion_params_from_pending_draw(
            pending_draw,
            has_previous_velocity_transform,
        ),
        flags,
        first_instance_index: entry.first_instance_index,
        instance_count: entry.instance_count,
        payload_slot,
    }
}

fn primitive_flags_for_renderer(
    common: &RendererCommon,
    has_previous_velocity_transform: bool,
) -> u32 {
    let mut flags = GPU_PRIMITIVE_FLAG_VISIBLE;
    if common.cast_shadows.casts_shadows() {
        flags |= GPU_PRIMITIVE_FLAG_CAST_SHADOWS;
    }
    if has_previous_velocity_transform {
        flags |= GPU_PRIMITIVE_FLAG_HAS_PREVIOUS_TRANSFORM;
    }
    flags
}

fn instance_data_for_pending_draw(
    pending_draw: &PendingMeshDraw,
    entry: GpuSceneEntry,
    previous_model_matrix: [[f32; 4]; 4],
    stable_instance_key: u64,
    lightmaps: Option<&LightmapConsumeContract>,
) -> GpuInstanceData {
    let payload_slot = virtual_geometry_payload_slot_for_pending_draw(pending_draw);
    let mut instance = GpuInstanceData {
        world_from_local: pending_draw.model_matrix,
        prev_world_from_local: previous_model_matrix,
        primitive_index: entry.primitive_index,
        flags: 0,
        payload_slot,
        morph_payload_slot: pending_draw
            .morph_payload_slot
            .unwrap_or(GPU_SCENE_INVALID_PAYLOAD_SLOT),
        lightmap_uv_rect: [0.0; 4],
        lightmap_params: [0; 4],
    };
    if let Some((contract, slot)) = lightmaps
        .filter(|_| pending_draw.mobility == Mobility::Static)
        .and_then(|contract| {
            contract
                .slot_for_instance(stable_instance_key)
                .map(|slot| (contract, slot))
        })
    {
        instance.set_lightmap(slot, contract.light_set_generation);
    }
    instance
}

fn virtual_geometry_payload_slot_for_pending_draw(pending_draw: &PendingMeshDraw) -> u32 {
    pending_draw
        .indirect_draw_ref
        .and_then(|draw_ref| draw_ref.segment_key.submission_slot)
        .unwrap_or(GPU_SCENE_INVALID_PAYLOAD_SLOT)
}

fn previous_model_matrix_for_gpu_scene_entry(
    gpu_scene: &GpuScene,
    pending_draw: &PendingMeshDraw,
    entry: GpuSceneEntry,
) -> ([[f32; 4]; 4], bool) {
    gpu_scene
        .previous_world_from_local(entry)
        .map(|previous| (previous, true))
        .unwrap_or((pending_draw.model_matrix, false))
}

fn velocity_history_is_available(is_skinned: bool, has_previous_transform: bool) -> bool {
    // A non-skinned first frame uses its current transform as the previous one,
    // which is a valid zero-velocity sample. Keeping that eligibility stable
    // avoids a primitive-data upload solely to flip the history-valid flag.
    !is_skinned || has_previous_transform
}

fn shadow_params_from_pending_draw(pending_draw: &PendingMeshDraw) -> [f32; 4] {
    let alpha_cutoff = pending_draw
        .pipeline_key
        .alpha_cutoff_bits
        .map(f32::from_bits)
        .filter(|cutoff| cutoff.is_finite())
        .unwrap_or(0.5)
        .clamp(0.0, 1.0);

    [
        if pending_draw.pipeline_key.is_alpha_mask() {
            1.0
        } else {
            0.0
        },
        alpha_cutoff,
        if pending_draw.common.receive_shadows {
            1.0
        } else {
            0.0
        },
        0.0,
    ]
}

fn motion_params_from_pending_draw(
    pending_draw: &PendingMeshDraw,
    has_previous_velocity_transform: bool,
) -> [f32; 4] {
    [
        if has_previous_velocity_transform {
            1.0
        } else {
            0.0
        },
        if pending_draw.skinned { 1.0 } else { 0.0 },
        if pending_draw.skinned
            && has_previous_velocity_transform
            && pending_draw.previous_skinned_joint_palette.is_some()
        {
            1.0
        } else {
            0.0
        },
        if pending_draw.pipeline_key.has_normal_texture {
            1.0
        } else {
            0.0
        },
    ]
}

fn approximate_transform_radius(model_matrix: &[[f32; 4]; 4]) -> f32 {
    let x = column_length(model_matrix[0]);
    let y = column_length(model_matrix[1]);
    let z = column_length(model_matrix[2]);
    x.max(y).max(z)
}

fn column_length(column: [f32; 4]) -> f32 {
    (column[0] * column[0] + column[1] * column[1] + column[2] * column[2]).sqrt()
}

fn resolved_skinned_gpu_source_for_pending_draw(
    device: &wgpu::Device,
    pending_draw: &PendingMeshDraw,
) -> Option<std::sync::Arc<GpuMeshResource>> {
    pending_draw
        .pipeline_key
        .uses_fallback_shader()
        .then_some(())?;
    Some(resolve_skinned_gpu_source_mesh(
        device,
        pending_draw.skinned_gpu_source.as_ref()?,
    ))
}

fn resolve_skinned_gpu_source_mesh(
    device: &wgpu::Device,
    source: &PendingSkinnedGpuSource,
) -> std::sync::Arc<GpuMeshResource> {
    match source {
        PendingSkinnedGpuSource::Prepared(mesh) => mesh.clone(),
        PendingSkinnedGpuSource::CpuMorphed { primitive, .. } => {
            std::sync::Arc::new(GpuMeshResource::from_asset(device, primitive.clone()))
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::core::framework::render::{CastShadowsMode, RendererCommon};
    use crate::graphics::scene::gpu_scene::{
        GPU_PRIMITIVE_FLAG_CAST_SHADOWS, GPU_PRIMITIVE_FLAG_HAS_PREVIOUS_TRANSFORM,
        GPU_PRIMITIVE_FLAG_VISIBLE,
    };

    use super::{primitive_flags_for_renderer, velocity_history_is_available};

    #[test]
    fn render_renderer_common_shadow_modes_project_to_gpu_primitive_flags() {
        let off = renderer_common(CastShadowsMode::Off);
        let two_sided = renderer_common(CastShadowsMode::TwoSided);
        let shadows_only = renderer_common(CastShadowsMode::ShadowsOnly);

        assert_eq!(
            primitive_flags_for_renderer(&off, false),
            GPU_PRIMITIVE_FLAG_VISIBLE
        );
        assert_ne!(
            primitive_flags_for_renderer(&two_sided, false) & GPU_PRIMITIVE_FLAG_CAST_SHADOWS,
            0
        );
        assert_ne!(
            primitive_flags_for_renderer(&shadows_only, true) & GPU_PRIMITIVE_FLAG_CAST_SHADOWS,
            0
        );
        assert_ne!(
            primitive_flags_for_renderer(&shadows_only, true)
                & GPU_PRIMITIVE_FLAG_HAS_PREVIOUS_TRANSFORM,
            0
        );
    }

    #[test]
    fn render_non_skinned_first_frame_seeds_zero_velocity_history() {
        assert!(velocity_history_is_available(false, false));
        assert!(!velocity_history_is_available(true, false));
        assert!(velocity_history_is_available(true, true));
    }

    fn renderer_common(cast_shadows: CastShadowsMode) -> RendererCommon {
        RendererCommon {
            cast_shadows,
            ..RendererCommon::default()
        }
    }
}

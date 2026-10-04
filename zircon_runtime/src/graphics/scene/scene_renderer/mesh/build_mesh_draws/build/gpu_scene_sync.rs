use std::collections::HashMap;

use crate::core::framework::render::{LightmapConsumeContract, RendererCommon};
use crate::core::framework::scene::Mobility;
use crate::core::math::RenderVec4;
use crate::graphics::backend::RenderBackend;
use crate::graphics::scene::gpu_scene::{
    GpuInstanceData, GpuPrimitiveData, GpuScene, GpuSceneEntry, GpuScenePreparedUpload,
    GPU_INSTANCE_FLAG_DEGENERATE_NORMAL_TRANSFORM, GPU_INSTANCE_FLAG_GENERAL_NORMAL_TRANSFORM,
    GPU_INSTANCE_FLAG_NEGATIVE_DETERMINANT, GPU_INSTANCE_FLAG_NON_ORTHOGONAL_TRANSFORM,
    GPU_PRIMITIVE_FLAG_CAST_SHADOWS, GPU_PRIMITIVE_FLAG_FORCE_HZB_VISIBLE,
    GPU_PRIMITIVE_FLAG_HAS_PREVIOUS_TRANSFORM, GPU_PRIMITIVE_FLAG_VISIBLE,
    GPU_SCENE_INVALID_PAYLOAD_SLOT,
};
use crate::graphics::scene::resources::GpuMeshResource;
use crate::graphics::types::GraphicsError;

use super::super::super::super::primitives::render_vec4_or;
use super::super::MeshHitProxyTokenSource;
use super::geometry_source_selection::pending_draw_has_enabled_skinned_gpu_source;
use super::gpu_scene_bounds::project_local_bounds_for_gpu_scene;
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

/// 材质与几何选择完成后、缓存抽取前同步稳定实例；返回待成功链确认的上传，上一帧变形历史此时仍未滚动。
pub(super) fn sync_gpu_scene_pending_draws(
    backend: &RenderBackend,
    encoder: &mut wgpu::CommandEncoder,
    gpu_scene: &mut GpuScene,
    pending_draws: &mut [PendingMeshDraw],
    virtual_geometry_counts: [u32; 2],
    lightmaps: Option<&LightmapConsumeContract>,
    hit_proxy_tokens: Option<&dyn MeshHitProxyTokenSource>,
) -> Result<(GpuScenePreparedUpload, HashMap<u64, SyncedGpuSceneEntry>), GraphicsError> {
    let device = &backend.device;
    let draw_capacity = pending_draws.len();
    let mut entries = HashMap::with_capacity(draw_capacity);
    gpu_scene.begin_skinned_joint_palette_frame();
    for pending_draw in pending_draws {
        let stable_instance_key = pending_draw.stable_instance_key;
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
        let skinned_gpu_skinning_enabled =
            pending_draw_has_enabled_skinned_gpu_source(pending_draw);
        gpu_scene.stage_current_skinned_joint_palette(
            stable_instance_key,
            skinned_joint_palette_state_for_pending_draw(pending_draw),
        );
        let skinning_palette_params = gpu_scene.stage_skinned_joint_palette_arena(
            stable_instance_key,
            pending_draw.skinned_joint_palette.as_ref(),
            skinned_gpu_skinning_enabled,
            skinned_gpu_skinning_enabled && pending_draw.previous_skinned_joint_palette.is_some(),
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
        let normal_transform_flags = pending_draw.normal_transform_flags;
        let hit_proxy_token = resolve_hit_proxy_token(stable_instance_key, hit_proxy_tokens);
        gpu_scene.write_primitive(
            entry,
            primitive_data_for_pending_draw(
                pending_draw,
                entry,
                has_previous_velocity_transform,
                skinned_gpu_skinning_enabled,
                hit_proxy_token,
            ),
        );
        gpu_scene.write_instances(
            entry,
            &[instance_data_for_pending_draw(
                pending_draw,
                entry,
                previous_model_matrix,
                stable_instance_key,
                lightmaps,
                normal_transform_flags,
                skinning_palette_params,
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
    if !gpu_scene.journal_membership_authoritative() {
        let live_keys = entries.keys().copied().collect();
        gpu_scene.retain_registered_keys(&live_keys);
    }
    let (palette_batch, palette_uploaded_bytes) =
        gpu_scene.prepare_skinned_joint_palette_upload(device);
    let mut prepared = gpu_scene.prepare_updates_with_staging_for_virtual_geometry_counts(
        backend,
        encoder,
        virtual_geometry_counts,
    );
    prepared.append_additional_upload(palette_batch, palette_uploaded_bytes);
    Ok((prepared, entries))
}

fn primitive_data_for_pending_draw(
    pending_draw: &PendingMeshDraw,
    entry: GpuSceneEntry,
    has_previous_velocity_transform: bool,
    skinned_gpu_skinning_enabled: bool,
    hit_proxy_token: u32,
) -> GpuPrimitiveData {
    let local_bounds = project_local_bounds_for_gpu_scene(
        pending_draw.local_bounds,
        pending_draw.hzb_bounds_are_temporally_stable(),
    );
    let flags = primitive_flags_for_renderer(
        pending_draw.material.common.as_ref(),
        has_previous_velocity_transform,
        local_bounds.force_hzb_visible,
    );
    let payload_slot = virtual_geometry_payload_slot_for_pending_draw(pending_draw);

    GpuPrimitiveData {
        local_bounds_center: local_bounds.center,
        local_bounds_radius: local_bounds.radius,
        tint: render_vec4_or(pending_draw.material.draw_tint, RenderVec4::ONE).to_array(),
        shadow_params: shadow_params_from_pending_draw(pending_draw),
        motion_params: motion_params_from_pending_draw(
            pending_draw,
            has_previous_velocity_transform,
            skinned_gpu_skinning_enabled,
        ),
        flags,
        first_instance_index: entry.first_instance_index,
        instance_count: entry.instance_count,
        payload_slot,
        material_payload_slot: GPU_SCENE_INVALID_PAYLOAD_SLOT,
        hit_proxy_token,
        material_payload_padding: [0; 2],
    }
}

fn resolve_hit_proxy_token(
    stable_instance_key: u64,
    source: Option<&dyn MeshHitProxyTokenSource>,
) -> u32 {
    source
        .and_then(|source| source.token_for_instance(stable_instance_key))
        .filter(|token| *token != 0)
        .unwrap_or(0)
}

fn primitive_flags_for_renderer(
    common: &RendererCommon,
    has_previous_velocity_transform: bool,
    force_hzb_visible: bool,
) -> u32 {
    let mut flags = GPU_PRIMITIVE_FLAG_VISIBLE;
    if common.cast_shadows.casts_shadows() {
        flags |= GPU_PRIMITIVE_FLAG_CAST_SHADOWS;
    }
    if has_previous_velocity_transform {
        flags |= GPU_PRIMITIVE_FLAG_HAS_PREVIOUS_TRANSFORM;
    }
    if force_hzb_visible {
        flags |= GPU_PRIMITIVE_FLAG_FORCE_HZB_VISIBLE;
    }
    flags
}

fn instance_data_for_pending_draw(
    pending_draw: &PendingMeshDraw,
    entry: GpuSceneEntry,
    previous_model_matrix: [[f32; 4]; 4],
    stable_instance_key: u64,
    lightmaps: Option<&LightmapConsumeContract>,
    normal_transform_flags: u32,
    skinning_palette_params: [u32; 4],
) -> GpuInstanceData {
    let payload_slot = virtual_geometry_payload_slot_for_pending_draw(pending_draw);
    let mut instance = GpuInstanceData {
        world_from_local: pending_draw.model_matrix,
        prev_world_from_local: previous_model_matrix,
        primitive_index: entry.primitive_index,
        flags: normal_transform_flags,
        payload_slot,
        morph_payload_slot: pending_draw
            .morph_payload_slot
            .unwrap_or(GPU_SCENE_INVALID_PAYLOAD_SLOT),
        lightmap_uv_rect: [0.0; 4],
        lightmap_params: [0; 4],
        skinning_palette_params,
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
        .material
        .pipeline_key
        .alpha_cutoff_bits
        .map(f32::from_bits)
        .filter(|cutoff| cutoff.is_finite())
        .unwrap_or(0.5)
        .clamp(0.0, 1.0);

    [
        if pending_draw.material.pipeline_key.is_alpha_mask() {
            1.0
        } else {
            0.0
        },
        alpha_cutoff,
        if pending_draw.material.common.receive_shadows {
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
    skinned_gpu_skinning_enabled: bool,
) -> [f32; 4] {
    [
        if has_previous_velocity_transform {
            1.0
        } else {
            0.0
        },
        if skinned_gpu_skinning_enabled {
            1.0
        } else {
            0.0
        },
        if skinned_gpu_skinning_enabled
            && has_previous_velocity_transform
            && pending_draw.previous_skinned_joint_palette.is_some()
        {
            1.0
        } else {
            0.0
        },
        if pending_draw.material.pipeline_key.has_normal_texture {
            1.0
        } else {
            0.0
        },
    ]
}

const NORMAL_TRANSFORM_MINIMUM_RELATIVE_DETERMINANT: f32 = 1.0e-6;
const NORMAL_TRANSFORM_ORTHOGONAL_RELATIVE_TOLERANCE: f32 = 1.0e-5;

/// Classifies the affine normal path once per changed instance instead of
/// expanding the 192-byte GPU scene ABI or inverting per vertex.
fn normal_transform_flags_for_model_matrix(model_matrix: &[[f32; 4]; 4]) -> u32 {
    let x = [model_matrix[0][0], model_matrix[0][1], model_matrix[0][2]];
    let y = [model_matrix[1][0], model_matrix[1][1], model_matrix[1][2]];
    let z = [model_matrix[2][0], model_matrix[2][1], model_matrix[2][2]];
    if !x
        .iter()
        .chain(y.iter())
        .chain(z.iter())
        .all(|component| component.is_finite())
    {
        return GPU_INSTANCE_FLAG_DEGENERATE_NORMAL_TRANSFORM;
    }

    let x_length_squared = dot3(x, x);
    let y_length_squared = dot3(y, y);
    let z_length_squared = dot3(z, z);
    let maximum_length_squared = x_length_squared.max(y_length_squared).max(z_length_squared);
    let maximum_axis_length = maximum_length_squared.sqrt();
    let determinant = dot3(x, cross3(y, z));
    let minimum_determinant = maximum_length_squared
        * maximum_axis_length
        * NORMAL_TRANSFORM_MINIMUM_RELATIVE_DETERMINANT;
    if !maximum_length_squared.is_finite()
        || !minimum_determinant.is_finite()
        || !determinant.is_finite()
        || determinant.abs() <= minimum_determinant
    {
        return GPU_INSTANCE_FLAG_DEGENERATE_NORMAL_TRANSFORM;
    }

    let tolerance = maximum_length_squared * NORMAL_TRANSFORM_ORTHOGONAL_RELATIVE_TOLERANCE;
    let orthogonal = dot3(x, y).abs() <= tolerance
        && dot3(y, z).abs() <= tolerance
        && dot3(z, x).abs() <= tolerance;
    let equal_length = (x_length_squared - y_length_squared).abs() <= tolerance
        && (y_length_squared - z_length_squared).abs() <= tolerance;

    let mut flags = if orthogonal && equal_length {
        0
    } else {
        GPU_INSTANCE_FLAG_GENERAL_NORMAL_TRANSFORM
    };
    if !orthogonal {
        flags |= GPU_INSTANCE_FLAG_NON_ORTHOGONAL_TRANSFORM;
    }
    if determinant.is_sign_negative() {
        flags |= GPU_INSTANCE_FLAG_NEGATIVE_DETERMINANT;
    }
    flags
}

pub(super) fn normal_transform_flags_for_pending_instance(
    gpu_scene: &GpuScene,
    stable_instance_key: u64,
    model_matrix: &[[f32; 4]; 4],
) -> u32 {
    gpu_scene
        .entry(stable_instance_key)
        .and_then(|entry| gpu_scene.instance_flags_for_world_from_local(entry, model_matrix))
        .unwrap_or_else(|| normal_transform_flags_for_model_matrix(model_matrix))
}

pub(super) fn normal_transform_flags_reverse_raster_winding(flags: u32) -> bool {
    flags & GPU_INSTANCE_FLAG_NEGATIVE_DETERMINANT != 0
}

fn dot3(left: [f32; 3], right: [f32; 3]) -> f32 {
    left[0] * right[0] + left[1] * right[1] + left[2] * right[2]
}

fn cross3(left: [f32; 3], right: [f32; 3]) -> [f32; 3] {
    [
        left[1] * right[2] - left[2] * right[1],
        left[2] * right[0] - left[0] * right[2],
        left[0] * right[1] - left[1] * right[0],
    ]
}

fn resolved_skinned_gpu_source_for_pending_draw(
    device: &wgpu::Device,
    pending_draw: &PendingMeshDraw,
) -> Option<std::sync::Arc<GpuMeshResource>> {
    pending_draw
        .material
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
#[path = "tests/gpu_scene_sync.rs"]
mod tests;

#[cfg(test)]
#[path = "gpu_scene_sync/tests/optimization_batch_ip_runtime626_tests.rs"]
mod optimization_batch_ip_runtime626_tests;

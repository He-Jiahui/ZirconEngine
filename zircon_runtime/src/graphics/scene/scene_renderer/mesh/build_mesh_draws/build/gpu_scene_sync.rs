use std::collections::{HashMap, HashSet};

use crate::core::framework::render::render_mesh_stable_instance_key;
use crate::core::math::RenderVec4;
use crate::graphics::scene::gpu_scene::{
    GpuInstanceData, GpuPrimitiveData, GpuScene, GpuSceneEntry, GpuSceneUploadReport,
    GPU_PRIMITIVE_FLAG_CAST_SHADOWS, GPU_PRIMITIVE_FLAG_HAS_PREVIOUS_TRANSFORM,
    GPU_PRIMITIVE_FLAG_VISIBLE, GPU_SCENE_INVALID_PAYLOAD_SLOT,
};
use crate::graphics::scene::resources::GpuMeshResource;

use super::super::super::super::primitives::render_vec4_or;
use super::super::super::mesh_draw::MeshDrawGeometrySource;
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
        let (previous_model_matrix, has_previous_velocity_transform) =
            previous_model_matrix_for_gpu_scene_entry(gpu_scene, pending_draw, entry);
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
    let mut flags = GPU_PRIMITIVE_FLAG_VISIBLE;
    if pending_draw.cast_shadows {
        flags |= GPU_PRIMITIVE_FLAG_CAST_SHADOWS;
    }
    if has_previous_velocity_transform {
        flags |= GPU_PRIMITIVE_FLAG_HAS_PREVIOUS_TRANSFORM;
    }

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
        payload_slot: GPU_SCENE_INVALID_PAYLOAD_SLOT,
    }
}

fn instance_data_for_pending_draw(
    pending_draw: &PendingMeshDraw,
    entry: GpuSceneEntry,
    previous_model_matrix: [[f32; 4]; 4],
) -> GpuInstanceData {
    GpuInstanceData {
        world_from_local: pending_draw.model_matrix,
        prev_world_from_local: previous_model_matrix,
        primitive_index: entry.primitive_index,
        flags: 0,
        payload_slot: GPU_SCENE_INVALID_PAYLOAD_SLOT,
        _pad0: 0,
    }
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
        if pending_draw.receive_shadows {
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

pub(super) fn skinned_gpu_source_geometry_source(
    source: &PendingSkinnedGpuSource,
) -> MeshDrawGeometrySource {
    match source {
        PendingSkinnedGpuSource::Prepared(_) => MeshDrawGeometrySource::Prepared,
        PendingSkinnedGpuSource::CpuMorphed { .. } => {
            MeshDrawGeometrySource::DynamicGpuSkinningSource
        }
    }
}

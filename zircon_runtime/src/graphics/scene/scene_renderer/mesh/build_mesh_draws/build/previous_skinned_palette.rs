use std::sync::Arc;

use crate::graphics::scene::gpu_scene::{
    GpuScene, GpuSceneSkinnedGpuSourceState, GpuSceneSkinnedJointPaletteState,
};
use crate::graphics::scene::resources::GpuMeshResource;
use crate::graphics::scene::scene_renderer::mesh::skinning::SkinnedMeshJointPaletteStorage;

use super::pending_mesh_draw::{PendingMeshDraw, PendingSkinnedGpuSource};

#[derive(Default)]
pub(super) struct PreviousSkinnedGpuState {
    pub(super) joint_palette: Option<SkinnedMeshJointPaletteStorage>,
    pub(super) source: Option<Arc<GpuMeshResource>>,
}

/// 从 GPUScene 历史选择速度 pass 可用的关节调色板与形变源。
/// 骨架签名、关节数量或 CPU 形变形状不兼容时丢弃历史。
pub(super) fn previous_skinned_gpu_state_for_gpu_scene_entry(
    gpu_scene: &GpuScene,
    stable_instance_key: u64,
    pending_draw: &PendingMeshDraw,
) -> PreviousSkinnedGpuState {
    let uses_cpu_morphed_source = matches!(
        pending_draw.skinned_gpu_source.as_ref(),
        Some(PendingSkinnedGpuSource::CpuMorphed { .. })
    );
    previous_skinned_gpu_state_for_states(
        uses_cpu_morphed_source,
        skinned_joint_palette_state_for_pending_draw(pending_draw),
        gpu_scene.previous_skinned_joint_palette_state(stable_instance_key),
        gpu_scene.previous_skinned_gpu_source_state(stable_instance_key),
    )
}

pub(super) fn skinned_joint_palette_state_for_pending_draw(
    pending_draw: &PendingMeshDraw,
) -> Option<GpuSceneSkinnedJointPaletteState> {
    Some(GpuSceneSkinnedJointPaletteState {
        signature: pending_draw.skinned_palette_signature?,
        morph_shape_signature: pending_draw
            .skinned_gpu_source
            .as_ref()
            .and_then(PendingSkinnedGpuSource::morph_shape_signature),
        storage: pending_draw.skinned_joint_palette?,
    })
}

pub(super) fn skinned_gpu_source_state_for_pending_draw(
    pending_draw: &PendingMeshDraw,
) -> Option<GpuSceneSkinnedGpuSourceState> {
    let source = pending_draw.skinned_gpu_source.as_ref()?;
    let PendingSkinnedGpuSource::CpuMorphed {
        morph_shape_signature,
        ..
    } = source
    else {
        return None;
    };
    Some(GpuSceneSkinnedGpuSourceState {
        morph_shape_signature: *morph_shape_signature,
        mesh: pending_draw.resolved_skinned_gpu_source.clone()?,
    })
}

fn previous_skinned_gpu_state_for_states(
    uses_cpu_morphed_source: bool,
    current: Option<GpuSceneSkinnedJointPaletteState>,
    previous: Option<GpuSceneSkinnedJointPaletteState>,
    previous_source: Option<GpuSceneSkinnedGpuSourceState>,
) -> PreviousSkinnedGpuState {
    let Some(current) = current else {
        return PreviousSkinnedGpuState::default();
    };
    let Some(previous) = previous else {
        return PreviousSkinnedGpuState::default();
    };
    if previous.signature != current.signature
        || previous.storage.joint_count() != current.storage.joint_count()
    {
        return PreviousSkinnedGpuState::default();
    }
    if !uses_cpu_morphed_source {
        return PreviousSkinnedGpuState {
            joint_palette: Some(previous.storage),
            source: None,
        };
    }

    let Some(current_morph_shape_signature) = current.morph_shape_signature else {
        return PreviousSkinnedGpuState::default();
    };
    if Some(current_morph_shape_signature) == previous.morph_shape_signature {
        return PreviousSkinnedGpuState {
            joint_palette: Some(previous.storage),
            source: None,
        };
    }

    let Some(previous_morph_shape_signature) = previous.morph_shape_signature else {
        return PreviousSkinnedGpuState::default();
    };
    let Some(previous_source) = previous_source else {
        return PreviousSkinnedGpuState::default();
    };
    if previous_source.morph_shape_signature != previous_morph_shape_signature {
        return PreviousSkinnedGpuState::default();
    }
    PreviousSkinnedGpuState {
        joint_palette: Some(previous.storage),
        source: Some(previous_source.mesh),
    }
}

#[cfg(test)]
#[path = "tests/previous_skinned_palette.rs"]
mod tests;

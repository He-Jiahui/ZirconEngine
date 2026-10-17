use std::sync::Arc;

use crate::core::math::Mat4;

pub(in crate::graphics::scene::scene_renderer::mesh) const SKINNED_MESH_MAX_JOINT_MATRICES: usize =
    256;

/// Immutable CPU snapshot. Only its matrix slice is uploaded to the GPU arena.
#[derive(Debug, PartialEq)]
pub(in crate::graphics::scene) struct SkinnedMeshJointPaletteStorage {
    joint_matrices: Box<[[[f32; 4]; 4]]>,
}

impl SkinnedMeshJointPaletteStorage {
    /// 超出快照容量返回 Err，供准备阶段选择 CPU 变形回退；不会截断关节矩阵或骨骼索引。
    pub(in crate::graphics::scene) fn from_matrices(matrices: &[Mat4]) -> Result<Self, String> {
        let joint_count = matrices.len();
        if joint_count > SKINNED_MESH_MAX_JOINT_MATRICES {
            return Err(format!(
                "skinned mesh joint palette has {joint_count} matrices, but the storage GPU ABI supports at most {SKINNED_MESH_MAX_JOINT_MATRICES}"
            ));
        }

        Ok(Self {
            joint_matrices: matrices.iter().map(Mat4::to_cols_array_2d).collect(),
        })
    }

    pub(in crate::graphics::scene) fn joint_count(&self) -> u32 {
        self.joint_matrices.len() as u32
    }

    pub(in crate::graphics::scene::scene_renderer::mesh) fn joint_matrices(
        &self,
    ) -> &[[[f32; 4]; 4]] {
        &self.joint_matrices
    }

    /// Only active matrices are uploaded; the snapshot header is not part of the WGSL layout.
    pub(in crate::graphics::scene) fn active_joint_matrices(&self) -> &[[[f32; 4]; 4]] {
        &self.joint_matrices
    }
}

pub(in crate::graphics::scene::scene_renderer) fn skinned_joint_palette_arena_min_binding_size(
) -> wgpu::BufferSize {
    wgpu::BufferSize::new(std::mem::size_of::<[[f32; 4]; 4]>() as u64)
        .unwrap_or(std::num::NonZeroU64::MIN)
}

pub(in crate::graphics::scene::scene_renderer) fn create_empty_skinned_joint_palette_arena_buffer(
    device: &wgpu::Device,
) -> Arc<wgpu::Buffer> {
    Arc::new(device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("zircon-skinned-joint-palette-arena-empty-buffer"),
        size: skinned_joint_palette_arena_min_binding_size().get(),
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    }))
}

#[cfg(test)]
#[path = "tests/joint_palette_storage.rs"]
mod tests;

#[cfg(all(test, feature = "memory-validation"))]
#[path = "tests/memory_profile.rs"]
mod memory_profile;

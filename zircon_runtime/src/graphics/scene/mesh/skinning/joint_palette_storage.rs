use std::sync::Arc;

use bytemuck::{Pod, Zeroable};

use crate::core::math::Mat4;

pub(in crate::graphics::scene::scene_renderer::mesh) const SKINNED_MESH_MAX_JOINT_MATRICES: usize =
    256;

/// Fixed CPU snapshot for one skinned entity's pose.
#[repr(C, align(16))]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub(in crate::graphics::scene) struct SkinnedMeshJointPaletteStorage {
    joint_matrices: [[[f32; 4]; 4]; SKINNED_MESH_MAX_JOINT_MATRICES],
    params: [u32; 4],
}

impl SkinnedMeshJointPaletteStorage {
    fn empty() -> Self {
        Self {
            joint_matrices: [Mat4::IDENTITY.to_cols_array_2d(); SKINNED_MESH_MAX_JOINT_MATRICES],
            params: [0; 4],
        }
    }

    /// 超出快照容量返回 Err，供准备阶段选择 CPU 变形回退；不会截断关节矩阵或骨骼索引。
    pub(in crate::graphics::scene) fn from_matrices(matrices: &[Mat4]) -> Result<Self, String> {
        let joint_count = matrices.len();
        if joint_count > SKINNED_MESH_MAX_JOINT_MATRICES {
            return Err(format!(
                "skinned mesh joint palette has {joint_count} matrices, but the storage GPU ABI supports at most {SKINNED_MESH_MAX_JOINT_MATRICES}"
            ));
        }

        let mut storage = Self::empty();
        for (index, matrix) in matrices.iter().enumerate() {
            storage.joint_matrices[index] = matrix.to_cols_array_2d();
        }
        storage.params[0] = joint_count as u32;
        Ok(storage)
    }

    pub(in crate::graphics::scene) fn joint_count(&self) -> u32 {
        self.params[0]
    }

    pub(in crate::graphics::scene::scene_renderer::mesh) fn joint_matrices(
        &self,
    ) -> &[[[f32; 4]; 4]; SKINNED_MESH_MAX_JOINT_MATRICES] {
        &self.joint_matrices
    }

    /// 供 arena 打包上传的活动矩阵前缀；CPU 快照尾部单位阵与 params 不属于 WGSL 的 palette 数组布局。
    pub(in crate::graphics::scene) fn active_joint_matrices(&self) -> &[[[f32; 4]; 4]] {
        let joint_count =
            usize::try_from(self.joint_count()).expect("skinned joint count did not fit usize");
        &self.joint_matrices[..joint_count]
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

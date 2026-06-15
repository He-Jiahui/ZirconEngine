use std::sync::Arc;

use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;

use crate::core::math::Mat4;

pub(in crate::graphics::scene::scene_renderer::mesh) const SKINNED_MESH_MAX_JOINT_MATRICES: usize =
    256;

#[repr(C, align(16))]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub(in crate::graphics::scene) struct SkinnedMeshJointPaletteUniform {
    joint_matrices: [[[f32; 4]; 4]; SKINNED_MESH_MAX_JOINT_MATRICES],
    params: [u32; 4],
}

impl SkinnedMeshJointPaletteUniform {
    pub(in crate::graphics::scene) fn from_matrices(matrices: &[Mat4]) -> Result<Self, String> {
        let joint_count = matrices.len();
        if joint_count > SKINNED_MESH_MAX_JOINT_MATRICES {
            return Err(format!(
                "skinned mesh joint palette has {joint_count} matrices, but the current uniform GPU ABI supports at most {SKINNED_MESH_MAX_JOINT_MATRICES}"
            ));
        }

        let mut joint_matrices =
            [Mat4::IDENTITY.to_cols_array_2d(); SKINNED_MESH_MAX_JOINT_MATRICES];
        for (index, matrix) in matrices.iter().enumerate() {
            joint_matrices[index] = matrix.to_cols_array_2d();
        }
        Ok(Self {
            joint_matrices,
            params: [joint_count as u32, 0, 0, 0],
        })
    }

    pub(in crate::graphics::scene) fn joint_count(&self) -> u32 {
        self.params[0]
    }

    pub(in crate::graphics::scene::scene_renderer::mesh) fn joint_matrices(
        &self,
    ) -> &[[[f32; 4]; 4]; SKINNED_MESH_MAX_JOINT_MATRICES] {
        &self.joint_matrices
    }

    pub(in crate::graphics::scene::scene_renderer::mesh) fn create_buffer(
        &self,
        device: &wgpu::Device,
    ) -> Arc<wgpu::Buffer> {
        Arc::new(
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("zircon-skinned-joint-palette-buffer"),
                contents: bytemuck::bytes_of(self),
                usage: wgpu::BufferUsages::UNIFORM,
            }),
        )
    }
}

pub(in crate::graphics::scene::scene_renderer) fn skinned_joint_palette_uniform_min_binding_size(
) -> wgpu::BufferSize {
    wgpu::BufferSize::new(std::mem::size_of::<SkinnedMeshJointPaletteUniform>() as u64)
        .expect("skinned joint palette uniform size is non-zero")
}

pub(in crate::graphics::scene::scene_renderer) fn create_empty_skinned_joint_palette_buffer(
    device: &wgpu::Device,
) -> Arc<wgpu::Buffer> {
    let uniform = SkinnedMeshJointPaletteUniform::from_matrices(&[])
        .expect("empty skinned joint palette fits the fixed uniform ABI");
    Arc::new(
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("zircon-empty-skinned-joint-palette-buffer"),
            contents: bytemuck::bytes_of(&uniform),
            usage: wgpu::BufferUsages::UNIFORM,
        }),
    )
}

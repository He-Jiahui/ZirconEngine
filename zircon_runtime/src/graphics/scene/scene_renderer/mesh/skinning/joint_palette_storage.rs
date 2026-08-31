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

    pub(in crate::graphics::scene) fn active_joint_matrices(&self) -> &[[[f32; 4]; 4]] {
        let joint_count =
            usize::try_from(self.joint_count()).expect("skinned joint count did not fit usize");
        &self.joint_matrices[..joint_count]
    }
}

pub(in crate::graphics::scene::scene_renderer) fn skinned_joint_palette_arena_min_binding_size()
-> wgpu::BufferSize {
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
mod tests {
    use super::*;

    const BENCHMARK_INSTANCE_COUNT: usize = 1_000;

    #[test]
    fn thousand_instance_storage_payload_contract_stays_within_expected_budget() {
        let payload = SkinnedMeshJointPaletteStorage::from_matrices(&[Mat4::IDENTITY; 64])
            .expect("64-joint test palette fits storage ABI");
        let payloads = vec![payload; BENCHMARK_INSTANCE_COUNT * 2];

        assert_eq!(payloads.len(), BENCHMARK_INSTANCE_COUNT * 2);
        assert_eq!(payloads[0].joint_count(), 64);
        assert_eq!(
            std::mem::size_of_val(payloads.as_slice()),
            BENCHMARK_INSTANCE_COUNT * 2 * std::mem::size_of::<SkinnedMeshJointPaletteStorage>()
        );
        assert!(std::mem::size_of_val(payloads.as_slice()) <= 32 * 1024 * 1024);
    }

    #[test]
    fn active_palette_span_exposes_only_live_joint_matrices() {
        let no_joints = SkinnedMeshJointPaletteStorage::from_matrices(&[])
            .expect("empty palette fits storage ABI");
        let sixty_four_joints =
            SkinnedMeshJointPaletteStorage::from_matrices(&[Mat4::IDENTITY; 64])
                .expect("64-joint palette fits storage ABI");
        let full_palette = SkinnedMeshJointPaletteStorage::from_matrices(
            &[Mat4::IDENTITY; SKINNED_MESH_MAX_JOINT_MATRICES],
        )
        .expect("full palette fits storage ABI");

        let matrix_bytes = std::mem::size_of::<[[f32; 4]; 4]>();
        assert_eq!(std::mem::size_of_val(no_joints.active_joint_matrices()), 0);
        assert_eq!(
            std::mem::size_of_val(sixty_four_joints.active_joint_matrices()),
            64 * matrix_bytes
        );
        assert_eq!(
            std::mem::size_of_val(full_palette.active_joint_matrices()),
            SKINNED_MESH_MAX_JOINT_MATRICES * matrix_bytes
        );
    }

    #[test]
    fn thousand_instance_two_frame_arena_payload_is_compact() {
        let matrix_bytes = std::mem::size_of::<[[f32; 4]; 4]>();
        let arena_payload_bytes = BENCHMARK_INSTANCE_COUNT * 2 * 64 * matrix_bytes;

        assert_eq!(arena_payload_bytes, 8_192_000);
        assert!(arena_payload_bytes < 8 * 1024 * 1024);
    }
}

use std::sync::Arc;

use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;

use crate::core::math::Mat4;

pub(in crate::graphics::scene::scene_renderer::mesh) const SKINNED_MESH_MAX_JOINT_MATRICES: usize =
    256;

/// Fixed GPU storage ABI for one skinned entity's current or previous pose.
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

    pub(in crate::graphics::scene) fn create_buffer(
        &self,
        device: &wgpu::Device,
    ) -> Arc<wgpu::Buffer> {
        Arc::new(
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("zircon-skinned-joint-palette-storage-buffer"),
                contents: bytemuck::bytes_of(self),
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            }),
        )
    }

    pub(in crate::graphics::scene) fn write_buffer(
        &self,
        queue: &wgpu::Queue,
        buffer: &wgpu::Buffer,
    ) {
        queue.write_buffer(buffer, 0, bytemuck::bytes_of(self));
    }
}

pub(in crate::graphics::scene::scene_renderer) fn skinned_joint_palette_storage_min_binding_size()
-> wgpu::BufferSize {
    wgpu::BufferSize::new(std::mem::size_of::<SkinnedMeshJointPaletteStorage>() as u64)
        .unwrap_or(std::num::NonZeroU64::MIN)
}

pub(in crate::graphics::scene::scene_renderer) fn create_empty_skinned_joint_palette_buffer(
    device: &wgpu::Device,
) -> Arc<wgpu::Buffer> {
    SkinnedMeshJointPaletteStorage::empty().create_buffer(device)
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
    #[ignore = "manual WGPU upload benchmark; run explicitly for M4 evidence"]
    fn gpu_skinning_storage_upload_benchmark_for_1000_instances() {
        let Ok(backend) = crate::graphics::backend::RenderBackend::new_offscreen() else {
            eprintln!("skipping GPU skinning upload benchmark: no offscreen adapter");
            return;
        };
        let payload = SkinnedMeshJointPaletteStorage::from_matrices(&[Mat4::IDENTITY; 64])
            .expect("64-joint benchmark palette fits storage ABI");
        let started = std::time::Instant::now();
        let buffers = (0..BENCHMARK_INSTANCE_COUNT * 2)
            .map(|_| payload.create_buffer(&backend.device))
            .collect::<Vec<_>>();

        assert_eq!(buffers.len(), BENCHMARK_INSTANCE_COUNT * 2);
        eprintln!(
            "gpu_skinning_storage_upload_1000_instances_elapsed_us={}",
            started.elapsed().as_micros()
        );
    }
}

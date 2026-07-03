use super::gpu_scene::{create_storage_buffer, GpuScene};
use super::layout::{
    GpuMorphDelta, GpuMorphPayload, GpuMorphWeight, GPU_MORPH_DELTA_STRIDE,
    GPU_MORPH_PAYLOAD_STRIDE, GPU_MORPH_WEIGHT_STRIDE,
};
use super::upload::write_full_pod_buffer;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct GpuSceneMorphUploadReport {
    pub(crate) payload_count: u32,
    pub(crate) delta_count: u32,
    pub(crate) weight_count: u32,
    pub(crate) uploaded_bytes: u64,
    pub(crate) rebuilt_bind_group: bool,
}

impl GpuScene {
    pub(crate) fn upload_morph_buffers(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        payloads: &[GpuMorphPayload],
        deltas: &[GpuMorphDelta],
        weights: &[GpuMorphWeight],
    ) -> GpuSceneMorphUploadReport {
        let payload_capacity_changed = self.morph_payloads_shadow.len() != payloads.len();
        let delta_capacity_changed = self.morph_deltas_shadow.len() != deltas.len();
        let weight_capacity_changed = self.morph_weights_shadow.len() != weights.len();

        if payload_capacity_changed {
            self.morph_payloads_buffer = create_storage_buffer(
                device,
                "zircon-gpu-scene-morph-payloads",
                buffer_size_for_len(payloads.len(), GPU_MORPH_PAYLOAD_STRIDE),
            );
        }
        if delta_capacity_changed {
            self.morph_deltas_buffer = create_storage_buffer(
                device,
                "zircon-gpu-scene-morph-deltas",
                buffer_size_for_len(deltas.len(), GPU_MORPH_DELTA_STRIDE),
            );
        }
        if weight_capacity_changed {
            self.morph_weights_buffer = create_storage_buffer(
                device,
                "zircon-gpu-scene-morph-weights",
                buffer_size_for_len(weights.len(), GPU_MORPH_WEIGHT_STRIDE),
            );
        }

        self.morph_payloads_shadow.clear();
        self.morph_payloads_shadow.extend_from_slice(payloads);
        self.morph_deltas_shadow.clear();
        self.morph_deltas_shadow.extend_from_slice(deltas);
        self.morph_weights_shadow.clear();
        self.morph_weights_shadow.extend_from_slice(weights);

        let uploaded_bytes = write_full_pod_buffer(
            queue,
            &self.morph_payloads_buffer,
            &self.morph_payloads_shadow,
            payloads.len(),
        ) + write_full_pod_buffer(
            queue,
            &self.morph_deltas_buffer,
            &self.morph_deltas_shadow,
            deltas.len(),
        ) + write_full_pod_buffer(
            queue,
            &self.morph_weights_buffer,
            &self.morph_weights_shadow,
            weights.len(),
        );
        let rebuilt_bind_group =
            payload_capacity_changed || delta_capacity_changed || weight_capacity_changed;
        if rebuilt_bind_group {
            self.rebuild_scene_bind_group(device);
        }

        GpuSceneMorphUploadReport {
            payload_count: u32::try_from(payloads.len()).unwrap_or(u32::MAX),
            delta_count: u32::try_from(deltas.len()).unwrap_or(u32::MAX),
            weight_count: u32::try_from(weights.len()).unwrap_or(u32::MAX),
            uploaded_bytes,
            rebuilt_bind_group,
        }
    }

    #[cfg(test)]
    pub(crate) fn debug_morph_payloads_shadow(&self) -> &[GpuMorphPayload] {
        &self.morph_payloads_shadow
    }

    #[cfg(test)]
    pub(crate) fn debug_morph_deltas_shadow(&self) -> &[GpuMorphDelta] {
        &self.morph_deltas_shadow
    }

    #[cfg(test)]
    pub(crate) fn debug_morph_weights_shadow(&self) -> &[GpuMorphWeight] {
        &self.morph_weights_shadow
    }
}

fn buffer_size_for_len(len: usize, stride: usize) -> u64 {
    let byte_len = len
        .checked_mul(stride)
        .and_then(|bytes| u64::try_from(bytes).ok())
        .unwrap_or(u64::MAX);
    byte_len.max(16)
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::graphics::scene::gpu_scene::{
        GpuMorphDelta, GpuMorphPayload, GpuMorphWeight, GpuScene,
    };

    const TEST_SKINNED_JOINT_MATRIX_COUNT: u64 = 256;
    const TEST_SKINNED_JOINT_MATRIX_BYTES: u64 = 64;
    const TEST_SKINNED_JOINT_PARAMS_BYTES: u64 = 16;

    #[test]
    fn render_gpu_scene_uploads_morph_storage_buffers() {
        let Some(backend) = test_backend() else {
            return;
        };
        let mut scene = test_gpu_scene(&backend.device);
        let payloads = [GpuMorphPayload::new(0, 0, 2, 1)];
        let deltas = [
            GpuMorphDelta::position_xyz(1.0, 2.0, 3.0),
            GpuMorphDelta::position_xyz(-1.0, 0.5, 0.25),
        ];
        let weights = [GpuMorphWeight::new(0.25), GpuMorphWeight::new(0.75)];

        let first_report = scene.upload_morph_buffers(
            &backend.device,
            &backend.queue,
            &payloads,
            &deltas,
            &weights,
        );

        assert_eq!(first_report.payload_count, 1);
        assert_eq!(first_report.delta_count, 2);
        assert_eq!(first_report.weight_count, 2);
        assert!(first_report.uploaded_bytes > 0);
        assert!(first_report.rebuilt_bind_group);
        assert_eq!(scene.debug_morph_payloads_shadow(), &payloads);
        assert_eq!(scene.debug_morph_deltas_shadow(), &deltas);
        assert_eq!(scene.debug_morph_weights_shadow(), &weights);

        let second_report = scene.upload_morph_buffers(
            &backend.device,
            &backend.queue,
            &payloads,
            &deltas,
            &weights,
        );

        assert_eq!(second_report.payload_count, 1);
        assert_eq!(second_report.delta_count, 2);
        assert_eq!(second_report.weight_count, 2);
        assert!(second_report.uploaded_bytes > 0);
        assert!(!second_report.rebuilt_bind_group);
    }

    fn test_backend() -> Option<crate::graphics::backend::RenderBackend> {
        crate::graphics::backend::RenderBackend::new_offscreen()
            .inspect_err(|error| eprintln!("skipping gpu scene morph upload test: {error:?}"))
            .ok()
    }

    fn test_gpu_scene(device: &wgpu::Device) -> GpuScene {
        GpuScene::new(
            device,
            test_skinned_joint_palette_buffer(device),
            test_skinned_joint_palette_min_binding_size(),
        )
    }

    fn test_skinned_joint_palette_buffer(device: &wgpu::Device) -> Arc<wgpu::Buffer> {
        Arc::new(device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("zircon-test-empty-skinned-joint-palette-buffer"),
            size: test_skinned_joint_palette_min_binding_size().get(),
            usage: wgpu::BufferUsages::UNIFORM,
            mapped_at_creation: false,
        }))
    }

    fn test_skinned_joint_palette_min_binding_size() -> wgpu::BufferSize {
        wgpu::BufferSize::new(
            TEST_SKINNED_JOINT_MATRIX_COUNT * TEST_SKINNED_JOINT_MATRIX_BYTES
                + TEST_SKINNED_JOINT_PARAMS_BYTES,
        )
        .expect("test skinned joint palette uniform size is non-zero")
    }
}

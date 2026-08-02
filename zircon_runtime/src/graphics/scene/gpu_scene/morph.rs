use super::gpu_scene::{
    create_storage_buffer, grow_capacity, GpuScene, GPU_SCENE_INITIAL_MORPH_CAPACITY,
};
use super::layout::{
    GpuMorphDelta, GpuMorphPayload, GpuMorphWeight, GPU_MORPH_DELTA_STRIDE,
    GPU_MORPH_PAYLOAD_STRIDE, GPU_MORPH_WEIGHT_STRIDE,
};
use super::upload::{write_changed_pod_buffer, write_full_pod_buffer};

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
        let payload_changed = self.morph_payloads_shadow != payloads;
        let delta_changed = self.morph_deltas_shadow != deltas;
        let weight_changed = self.morph_weights_shadow != weights;
        let required_payload_capacity =
            u32::try_from(payloads.len()).expect("morph payload buffer capacity exceeded u32");
        let required_delta_capacity =
            u32::try_from(deltas.len()).expect("morph delta buffer capacity exceeded u32");
        let required_weight_capacity =
            u32::try_from(weights.len()).expect("morph weight buffer capacity exceeded u32");
        let payload_buffer_replaced = required_payload_capacity > self.morph_payloads_capacity;
        let delta_buffer_replaced = required_delta_capacity > self.morph_deltas_capacity;
        let weight_buffer_replaced = required_weight_capacity > self.morph_weights_capacity;

        if payload_buffer_replaced {
            self.morph_payloads_capacity =
                grow_capacity(required_payload_capacity, GPU_SCENE_INITIAL_MORPH_CAPACITY);
            self.morph_payloads_buffer = create_storage_buffer(
                device,
                "zircon-gpu-scene-morph-payloads",
                buffer_size_for_len(
                    self.morph_payloads_capacity as usize,
                    GPU_MORPH_PAYLOAD_STRIDE,
                ),
            );
        }
        if delta_buffer_replaced {
            self.morph_deltas_capacity =
                grow_capacity(required_delta_capacity, GPU_SCENE_INITIAL_MORPH_CAPACITY);
            self.morph_deltas_buffer = create_storage_buffer(
                device,
                "zircon-gpu-scene-morph-deltas",
                buffer_size_for_len(self.morph_deltas_capacity as usize, GPU_MORPH_DELTA_STRIDE),
            );
        }
        if weight_buffer_replaced {
            self.morph_weights_capacity =
                grow_capacity(required_weight_capacity, GPU_SCENE_INITIAL_MORPH_CAPACITY);
            self.morph_weights_buffer = create_storage_buffer(
                device,
                "zircon-gpu-scene-morph-weights",
                buffer_size_for_len(
                    self.morph_weights_capacity as usize,
                    GPU_MORPH_WEIGHT_STRIDE,
                ),
            );
        }

        let uploaded_bytes = if payload_changed {
            if payload_buffer_replaced {
                write_full_pod_buffer(queue, &self.morph_payloads_buffer, payloads, payloads.len())
            } else {
                write_changed_pod_buffer(
                    queue,
                    &self.morph_payloads_buffer,
                    &self.morph_payloads_shadow,
                    payloads,
                )
            }
        } else {
            0
        } + if delta_changed {
            if delta_buffer_replaced {
                write_full_pod_buffer(queue, &self.morph_deltas_buffer, deltas, deltas.len())
            } else {
                write_changed_pod_buffer(
                    queue,
                    &self.morph_deltas_buffer,
                    &self.morph_deltas_shadow,
                    deltas,
                )
            }
        } else {
            0
        } + if weight_changed {
            if weight_buffer_replaced {
                write_full_pod_buffer(queue, &self.morph_weights_buffer, weights, weights.len())
            } else {
                write_changed_pod_buffer(
                    queue,
                    &self.morph_weights_buffer,
                    &self.morph_weights_shadow,
                    weights,
                )
            }
        } else {
            0
        };

        if payload_changed {
            self.morph_payloads_shadow.clear();
            self.morph_payloads_shadow.extend_from_slice(payloads);
        }
        if delta_changed {
            self.morph_deltas_shadow.clear();
            self.morph_deltas_shadow.extend_from_slice(deltas);
        }
        if weight_changed {
            self.morph_weights_shadow.clear();
            self.morph_weights_shadow.extend_from_slice(weights);
        }

        let rebuilt_bind_group =
            payload_buffer_replaced || delta_buffer_replaced || weight_buffer_replaced;
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
        GpuMorphDelta, GpuMorphPayload, GpuMorphWeight, GpuScene, GPU_MORPH_DELTA_STRIDE,
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
        assert_eq!(second_report.uploaded_bytes, 0);
        assert!(!second_report.rebuilt_bind_group);
    }

    #[test]
    fn render_gpu_scene_reuses_morph_buffers_when_active_rows_shrink() {
        let Some(backend) = test_backend() else {
            return;
        };
        let mut scene = test_gpu_scene(&backend.device);
        let payloads = [
            GpuMorphPayload::new(0, 0, 1, 1),
            GpuMorphPayload::new(1, 2, 1, 1),
        ];
        let deltas = [
            GpuMorphDelta::position_xyz(1.0, 2.0, 3.0),
            GpuMorphDelta::position_xyz(4.0, 5.0, 6.0),
        ];
        let weights = [GpuMorphWeight::new(0.25), GpuMorphWeight::new(0.75)];

        let first_report = scene.upload_morph_buffers(
            &backend.device,
            &backend.queue,
            &payloads,
            &deltas,
            &weights,
        );
        assert!(first_report.rebuilt_bind_group);

        let shrunk_payloads = [GpuMorphPayload::new(0, 0, 1, 1)];
        let shrunk_deltas = [GpuMorphDelta::position_xyz(7.0, 8.0, 9.0)];
        let shrunk_weights = [GpuMorphWeight::new(0.5)];
        let shrunk_report = scene.upload_morph_buffers(
            &backend.device,
            &backend.queue,
            &shrunk_payloads,
            &shrunk_deltas,
            &shrunk_weights,
        );

        assert!(shrunk_report.uploaded_bytes > 0);
        assert!(
            !shrunk_report.rebuilt_bind_group,
            "smaller morph payloads must reuse the existing storage buffers and scene bind group"
        );
        assert_eq!(scene.debug_morph_payloads_shadow(), &shrunk_payloads);
        assert_eq!(scene.debug_morph_deltas_shadow(), &shrunk_deltas);
        assert_eq!(scene.debug_morph_weights_shadow(), &shrunk_weights);
    }

    #[test]
    fn render_gpu_scene_uploads_only_changed_morph_delta_rows() {
        let Some(backend) = test_backend() else {
            return;
        };
        let mut scene = test_gpu_scene(&backend.device);
        let payloads = [GpuMorphPayload::new(0, 0, 1, 1)];
        let initial_deltas = [
            GpuMorphDelta::position_xyz(1.0, 2.0, 3.0),
            GpuMorphDelta::position_xyz(4.0, 5.0, 6.0),
        ];
        let weights = [GpuMorphWeight::new(0.25)];
        let _ = scene.upload_morph_buffers(
            &backend.device,
            &backend.queue,
            &payloads,
            &initial_deltas,
            &weights,
        );

        let changed_deltas = [
            initial_deltas[0],
            GpuMorphDelta::position_xyz(7.0, 8.0, 9.0),
        ];
        let report = scene.upload_morph_buffers(
            &backend.device,
            &backend.queue,
            &payloads,
            &changed_deltas,
            &weights,
        );

        assert_eq!(
            report.uploaded_bytes, GPU_MORPH_DELTA_STRIDE as u64,
            "one changed morph delta must upload exactly one storage row"
        );
        assert!(!report.rebuilt_bind_group);
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
            usage: wgpu::BufferUsages::STORAGE,
            mapped_at_creation: false,
        }))
    }

    fn test_skinned_joint_palette_min_binding_size() -> wgpu::BufferSize {
        wgpu::BufferSize::new(
            TEST_SKINNED_JOINT_MATRIX_COUNT * TEST_SKINNED_JOINT_MATRIX_BYTES
                + TEST_SKINNED_JOINT_PARAMS_BYTES,
        )
        .expect("test skinned joint palette storage size is non-zero")
    }
}

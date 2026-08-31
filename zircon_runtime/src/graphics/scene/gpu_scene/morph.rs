use super::gpu_scene::{
    GPU_SCENE_INITIAL_MORPH_CAPACITY, GpuScene, create_storage_buffer, grow_capacity,
};
use super::layout::{
    GPU_MORPH_DELTA_STRIDE, GPU_MORPH_PAYLOAD_STRIDE, GPU_MORPH_WEIGHT_STRIDE, GpuMorphDelta,
    GpuMorphPayload, GpuMorphWeight,
};
use super::upload::GpuSceneBufferUploadBatchBuilder;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use zr_rhi_wgpu::WgpuBufferUploadBatch;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct GpuSceneMorphUploadReport {
    pub(crate) payload_count: u32,
    pub(crate) delta_count: u32,
    pub(crate) weight_count: u32,
    pub(crate) uploaded_bytes: u64,
    pub(crate) rebuilt_bind_group: bool,
}

pub(crate) struct GpuScenePreparedMorphUpload {
    pub(super) owner: Arc<()>,
    pub(super) batch: WgpuBufferUploadBatch,
    pub(super) report: GpuSceneMorphUploadReport,
    pub(super) commit: GpuSceneMorphUploadCommit,
}

pub(super) struct GpuSceneMorphUploadCommit {
    payloads: Vec<GpuMorphPayload>,
    deltas: Vec<GpuMorphDelta>,
    weights: Vec<GpuMorphWeight>,
    reservation: GpuSceneMorphPreparationReservation,
}

struct GpuSceneMorphPreparationReservation {
    state: Arc<AtomicBool>,
}

impl GpuScenePreparedMorphUpload {
    pub(crate) const fn report(&self) -> GpuSceneMorphUploadReport {
        self.report
    }

    pub(super) fn is_owned_by(&self, owner: &Arc<()>) -> bool {
        Arc::ptr_eq(&self.owner, owner)
    }
}

impl GpuSceneMorphUploadCommit {
    pub(super) fn commit(self, gpu_scene: &mut GpuScene) {
        let Self {
            payloads,
            deltas,
            weights,
            reservation: _reservation,
        } = self;
        gpu_scene.morph_payloads_shadow = payloads;
        gpu_scene.morph_deltas_shadow = deltas;
        gpu_scene.morph_weights_shadow = weights;
        gpu_scene.morph_payloads_require_full_upload = false;
        gpu_scene.morph_deltas_require_full_upload = false;
        gpu_scene.morph_weights_require_full_upload = false;
    }
}

impl Drop for GpuSceneMorphPreparationReservation {
    fn drop(&mut self) {
        self.state.store(false, Ordering::Release);
    }
}

impl GpuScene {
    pub(crate) fn prepare_morph_buffers(
        &mut self,
        device: &wgpu::Device,
        payloads: Vec<GpuMorphPayload>,
        deltas: Vec<GpuMorphDelta>,
        weights: Vec<GpuMorphWeight>,
    ) -> GpuScenePreparedMorphUpload {
        let reservation = self.reserve_morph_upload_preparation();
        let payload_changed =
            self.morph_payloads_require_full_upload || self.morph_payloads_shadow != payloads;
        let delta_changed =
            self.morph_deltas_require_full_upload || self.morph_deltas_shadow != deltas;
        let weight_changed =
            self.morph_weights_require_full_upload || self.morph_weights_shadow != weights;
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
            self.morph_payloads_require_full_upload = true;
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
            self.morph_deltas_require_full_upload = true;
            self.morph_deltas_capacity =
                grow_capacity(required_delta_capacity, GPU_SCENE_INITIAL_MORPH_CAPACITY);
            self.morph_deltas_buffer = create_storage_buffer(
                device,
                "zircon-gpu-scene-morph-deltas",
                buffer_size_for_len(self.morph_deltas_capacity as usize, GPU_MORPH_DELTA_STRIDE),
            );
        }
        if weight_buffer_replaced {
            self.morph_weights_require_full_upload = true;
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

        let mut uploads = GpuSceneBufferUploadBatchBuilder::new();
        let uploaded_bytes = if payload_changed {
            if self.morph_payloads_require_full_upload {
                uploads.push_pod_slice(&self.morph_payloads_buffer, 0, &payloads)
            } else {
                uploads.push_changed_pod_slice(
                    &self.morph_payloads_buffer,
                    &self.morph_payloads_shadow,
                    &payloads,
                )
            }
        } else {
            0
        } + if delta_changed {
            if self.morph_deltas_require_full_upload {
                uploads.push_pod_slice(&self.morph_deltas_buffer, 0, &deltas)
            } else {
                uploads.push_changed_pod_slice(
                    &self.morph_deltas_buffer,
                    &self.morph_deltas_shadow,
                    &deltas,
                )
            }
        } else {
            0
        } + if weight_changed {
            if self.morph_weights_require_full_upload {
                uploads.push_pod_slice(&self.morph_weights_buffer, 0, &weights)
            } else {
                uploads.push_changed_pod_slice(
                    &self.morph_weights_buffer,
                    &self.morph_weights_shadow,
                    &weights,
                )
            }
        } else {
            0
        };

        let rebuilt_bind_group =
            payload_buffer_replaced || delta_buffer_replaced || weight_buffer_replaced;
        if rebuilt_bind_group {
            self.rebuild_scene_bind_group(device);
        }

        GpuScenePreparedMorphUpload {
            owner: Arc::clone(&self.upload_transaction_owner),
            batch: uploads.into_batch(),
            report: GpuSceneMorphUploadReport {
                payload_count: u32::try_from(payloads.len()).unwrap_or(u32::MAX),
                delta_count: u32::try_from(deltas.len()).unwrap_or(u32::MAX),
                weight_count: u32::try_from(weights.len()).unwrap_or(u32::MAX),
                uploaded_bytes,
                rebuilt_bind_group,
            },
            commit: GpuSceneMorphUploadCommit {
                payloads,
                deltas,
                weights,
                reservation,
            },
        }
    }

    fn reserve_morph_upload_preparation(&self) -> GpuSceneMorphPreparationReservation {
        let reserved = self
            .morph_preparation_reservation
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_ok();
        assert!(
            reserved,
            "GPU Scene permits one outstanding morph preparation"
        );
        GpuSceneMorphPreparationReservation {
            state: Arc::clone(&self.morph_preparation_reservation),
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
        GPU_MORPH_DELTA_STRIDE, GpuMorphDelta, GpuMorphPayload, GpuMorphWeight, GpuScene,
        GpuSceneMorphUploadReport,
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

        let first_report = submit_morph_upload(&mut scene, &backend, &payloads, &deltas, &weights);

        assert_eq!(first_report.payload_count, 1);
        assert_eq!(first_report.delta_count, 2);
        assert_eq!(first_report.weight_count, 2);
        assert!(first_report.uploaded_bytes > 0);
        assert!(first_report.rebuilt_bind_group);
        assert_eq!(scene.debug_morph_payloads_shadow(), &payloads);
        assert_eq!(scene.debug_morph_deltas_shadow(), &deltas);
        assert_eq!(scene.debug_morph_weights_shadow(), &weights);

        let second_report = submit_morph_upload(&mut scene, &backend, &payloads, &deltas, &weights);

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

        let first_report = submit_morph_upload(&mut scene, &backend, &payloads, &deltas, &weights);
        assert!(first_report.rebuilt_bind_group);

        let shrunk_payloads = [GpuMorphPayload::new(0, 0, 1, 1)];
        let shrunk_deltas = [GpuMorphDelta::position_xyz(7.0, 8.0, 9.0)];
        let shrunk_weights = [GpuMorphWeight::new(0.5)];
        let shrunk_report = submit_morph_upload(
            &mut scene,
            &backend,
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
        let _ = submit_morph_upload(&mut scene, &backend, &payloads, &initial_deltas, &weights);

        let changed_deltas = [
            initial_deltas[0],
            GpuMorphDelta::position_xyz(7.0, 8.0, 9.0),
        ];
        let report =
            submit_morph_upload(&mut scene, &backend, &payloads, &changed_deltas, &weights);

        assert_eq!(
            report.uploaded_bytes, GPU_MORPH_DELTA_STRIDE as u64,
            "one changed morph delta must upload exactly one storage row"
        );
        assert!(!report.rebuilt_bind_group);
    }

    #[test]
    fn render_gpu_scene_dropped_morph_preparation_keeps_committed_shadow_for_retry() {
        let Some(backend) = test_backend() else {
            return;
        };
        let mut scene = test_gpu_scene(&backend.device);
        let payloads = [GpuMorphPayload::new(0, 0, 1, 1)];
        let deltas = [GpuMorphDelta::position_xyz(1.0, 2.0, 3.0)];
        let weights = [GpuMorphWeight::new(0.5)];
        submit_morph_upload(&mut scene, &backend, &payloads, &deltas, &weights);
        let grown_payloads =
            vec![GpuMorphPayload::new(0, 0, 1, 1); scene.morph_payloads_capacity as usize + 1];
        let dropped = scene.prepare_morph_buffers(
            &backend.device,
            grown_payloads,
            deltas.to_vec(),
            weights.to_vec(),
        );
        assert!(dropped.report().uploaded_bytes > 0);
        assert!(dropped.report().rebuilt_bind_group);
        let mut dropped_frame = scene.prepare_direct_updates();
        dropped_frame.append_morph_upload(dropped);
        drop(dropped_frame);
        assert_eq!(scene.debug_morph_payloads_shadow(), &payloads);

        let retry = scene.prepare_morph_buffers(
            &backend.device,
            payloads.to_vec(),
            deltas.to_vec(),
            weights.to_vec(),
        );
        assert!(retry.report().uploaded_bytes > 0);
    }

    #[test]
    fn render_gpu_scene_rejects_overlapping_morph_preparations() {
        let Some(backend) = test_backend() else {
            return;
        };
        let mut scene = test_gpu_scene(&backend.device);
        let first = scene.prepare_morph_buffers(
            &backend.device,
            vec![GpuMorphPayload::new(0, 0, 1, 1)],
            vec![GpuMorphDelta::position_xyz(1.0, 2.0, 3.0)],
            vec![GpuMorphWeight::new(0.5)],
        );

        let overlapping = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            scene.prepare_morph_buffers(&backend.device, Vec::new(), Vec::new(), Vec::new())
        }));
        assert!(overlapping.is_err());

        drop(first);
        let retry =
            scene.prepare_morph_buffers(&backend.device, Vec::new(), Vec::new(), Vec::new());
        drop(retry);
    }

    #[test]
    fn render_gpu_scene_rejects_foreign_morph_preparation_attachment() {
        let Some(backend) = test_backend() else {
            return;
        };
        let mut source_scene = test_gpu_scene(&backend.device);
        let mut target_scene = test_gpu_scene(&backend.device);
        let prepared = source_scene.prepare_morph_buffers(
            &backend.device,
            vec![GpuMorphPayload::new(0, 0, 1, 1)],
            vec![GpuMorphDelta::position_xyz(1.0, 2.0, 3.0)],
            vec![GpuMorphWeight::new(0.5)],
        );
        let mut target_frame = target_scene.prepare_direct_updates();

        let attachment = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            target_frame.append_morph_upload(prepared);
        }));

        assert!(attachment.is_err());
        drop(target_frame);
        let retry =
            source_scene.prepare_morph_buffers(&backend.device, Vec::new(), Vec::new(), Vec::new());
        drop(retry);
    }

    fn submit_morph_upload(
        scene: &mut GpuScene,
        backend: &crate::graphics::backend::RenderBackend,
        payloads: &[GpuMorphPayload],
        deltas: &[GpuMorphDelta],
        weights: &[GpuMorphWeight],
    ) -> GpuSceneMorphUploadReport {
        let prepared = scene.prepare_morph_buffers(
            &backend.device,
            payloads.to_vec(),
            deltas.to_vec(),
            weights.to_vec(),
        );
        let report = prepared.report();
        let mut frame = scene.prepare_direct_updates();
        frame.append_morph_upload(prepared);
        scene
            .submit_prepared_upload(backend, frame)
            .expect("morph upload batch must be accepted by the test backend");
        report
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

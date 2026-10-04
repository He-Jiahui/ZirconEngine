use super::gpu_scene::{
    create_storage_buffer, grow_capacity, GpuScene, GPU_SCENE_INITIAL_MORPH_CAPACITY,
};
use super::layout::{
    GpuMorphDelta, GpuMorphPayload, GpuMorphWeight, GPU_MORPH_DELTA_STRIDE,
    GPU_MORPH_PAYLOAD_STRIDE, GPU_MORPH_WEIGHT_STRIDE,
};
use super::upload::GpuSceneBufferUploadBatchBuilder;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
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
#[path = "tests/morph.rs"]
mod tests;

use crate::core::framework::render::GpuLightData;

use super::gpu_scene::{GpuScene, GpuSceneUploadPath, GpuSceneUploadReport};
use super::layout::{GPU_INSTANCE_DATA_STRIDE, GPU_PRIMITIVE_DATA_STRIDE};
use super::staging_ring::{GpuSceneStagingDestination, GpuSceneStagingRing};

impl GpuScene {
    /// Uses the persistent frame ring once merged scene updates exceed the
    /// direct-write threshold. Small and empty updates retain the lower-overhead
    /// queue-write path used by unit-level GPUScene callers.
    pub(crate) fn flush_updates_with_staging(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
    ) -> GpuSceneUploadReport {
        if !GpuSceneStagingRing::should_stage(self.pending_upload_byte_len()) {
            return self.flush_direct_updates(queue);
        }

        self.flush_staged_updates(device, queue, encoder)
    }

    fn flush_staged_updates(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
    ) -> GpuSceneUploadReport {
        let dirty_entry_count = self.updates.dirty_entry_count();
        let mut report = GpuSceneUploadReport {
            upload_path: GpuSceneUploadPath::StagingCopy,
            ..GpuSceneUploadReport::default()
        };
        self.staging_ring.begin_frame();

        if self.force_full_primitive_upload {
            let active_len = self.primitive_ids.high_water() as usize;
            let uploaded = self.staging_ring.stage_pod_slice(
                GpuSceneStagingDestination::Primitive,
                0,
                &self.primitive_shadow[..active_len],
            );
            if uploaded > 0 {
                report.primitive_upload_range_count = 1;
                report.uploaded_bytes += uploaded;
            }
            self.updates.discard_primitive_updates();
        } else {
            let ranges = self
                .updates
                .drain_primitive_upload_ranges(GPU_PRIMITIVE_DATA_STRIDE as u64);
            report.primitive_upload_range_count = ranges.len();
            for range in ranges {
                let start = range.start as usize;
                let end = start
                    .checked_add(range.len as usize)
                    .expect("gpu scene primitive staging range overflowed usize");
                report.uploaded_bytes += self.staging_ring.stage_pod_slice(
                    GpuSceneStagingDestination::Primitive,
                    range.byte_offset,
                    &self.primitive_shadow[start..end],
                );
            }
        }

        if self.force_full_instance_upload {
            let active_len = self.instance_ids.high_water() as usize;
            let uploaded = self.staging_ring.stage_pod_slice(
                GpuSceneStagingDestination::Instance,
                0,
                &self.instance_shadow[..active_len],
            );
            if uploaded > 0 {
                report.instance_upload_range_count = 1;
                report.uploaded_bytes += uploaded;
            }
            self.updates.discard_instance_updates();
        } else {
            let ranges = self
                .updates
                .drain_instance_upload_ranges(GPU_INSTANCE_DATA_STRIDE as u64);
            report.instance_upload_range_count = ranges.len();
            for range in ranges {
                let start = range.start as usize;
                let end = start
                    .checked_add(range.len as usize)
                    .expect("gpu scene instance staging range overflowed usize");
                report.uploaded_bytes += self.staging_ring.stage_pod_slice(
                    GpuSceneStagingDestination::Instance,
                    range.byte_offset,
                    &self.instance_shadow[start..end],
                );
            }
        }

        if self.force_full_light_upload {
            let uploaded = self.staging_ring.stage_pod_slice(
                GpuSceneStagingDestination::Light,
                0,
                &self.light_shadow,
            );
            if uploaded > 0 {
                report.light_upload_range_count = 1;
                report.uploaded_bytes += uploaded;
            }
            self.updates.discard_light_updates();
        } else {
            let ranges = self
                .updates
                .drain_light_upload_ranges(GpuLightData::STRIDE as u64);
            report.light_upload_range_count = ranges.len();
            for range in ranges {
                let start = range.start as usize;
                let end = start
                    .checked_add(range.len as usize)
                    .expect("gpu scene light staging range overflowed usize");
                report.uploaded_bytes += self.staging_ring.stage_pod_slice(
                    GpuSceneStagingDestination::Light,
                    range.byte_offset,
                    &self.light_shadow[start..end],
                );
            }
        }

        self.staging_ring.submit(
            device,
            queue,
            encoder,
            &self.primitive_buffer,
            &self.instance_buffer,
            &self.light_buffer,
        );
        self.write_scene_data_count_params_if_needed(queue);

        self.force_full_primitive_upload = false;
        self.force_full_instance_upload = false;
        self.force_full_light_upload = false;
        self.primitive_ids.commit_pending_frees();
        self.instance_ids.commit_pending_frees();
        self.refresh_stats(report.uploaded_bytes, dirty_entry_count);
        report
    }

    fn pending_upload_byte_len(&mut self) -> u64 {
        let primitive_bytes = if self.force_full_primitive_upload {
            u64::from(self.primitive_ids.high_water()) * GPU_PRIMITIVE_DATA_STRIDE as u64
        } else {
            self.updates
                .prepare_primitive_upload_ranges(GPU_PRIMITIVE_DATA_STRIDE as u64)
                .iter()
                .map(|range| range.byte_len)
                .sum()
        };
        let instance_bytes = if self.force_full_instance_upload {
            u64::from(self.instance_ids.high_water()) * GPU_INSTANCE_DATA_STRIDE as u64
        } else {
            self.updates
                .prepare_instance_upload_ranges(GPU_INSTANCE_DATA_STRIDE as u64)
                .iter()
                .map(|range| range.byte_len)
                .sum()
        };
        let light_bytes = if self.force_full_light_upload {
            u64::try_from(self.light_shadow.len()).expect("gpu scene light count exceeded u64")
                * GpuLightData::STRIDE as u64
        } else {
            self.updates
                .prepare_light_upload_ranges(GpuLightData::STRIDE as u64)
                .iter()
                .map(|range| range.byte_len)
                .sum()
        };

        primitive_bytes
            .saturating_add(instance_bytes)
            .saturating_add(light_bytes)
    }
}

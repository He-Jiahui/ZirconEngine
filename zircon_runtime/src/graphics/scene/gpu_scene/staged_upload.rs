use crate::core::framework::render::GpuLightData;
use crate::graphics::backend::RenderBackend;
use crate::graphics::types::GraphicsError;

use super::gpu_scene::{GpuScene, GpuSceneUploadPath, GpuSceneUploadReport};
use super::layout::{GPU_INSTANCE_DATA_STRIDE, GPU_PRIMITIVE_DATA_STRIDE};
use super::prepared_upload::GpuScenePreparedUpload;
use super::staging_ring::{GpuSceneStagingDestination, GpuSceneStagingRing};
use super::upload::GpuSceneBufferUploadBatchBuilder;

impl GpuScene {
    /// Uses the persistent frame ring once merged scene updates exceed the
    /// direct-write threshold. Small and empty updates retain the lower-overhead
    /// batched buffer-write path used by unit-level GPUScene callers.
    pub(crate) fn flush_updates_with_staging(
        &mut self,
        backend: &RenderBackend,
        encoder: &mut wgpu::CommandEncoder,
    ) -> Result<GpuSceneUploadReport, GraphicsError> {
        let prepared = self.prepare_updates_with_staging(backend, encoder);
        self.submit_prepared_upload(backend, prepared)
    }

    pub(crate) fn prepare_updates_with_staging(
        &mut self,
        backend: &RenderBackend,
        encoder: &mut wgpu::CommandEncoder,
    ) -> GpuScenePreparedUpload {
        let scene_data_counts = self.current_scene_data_counts();
        self.prepare_updates_with_staging_for_scene_data_counts(backend, encoder, scene_data_counts)
    }

    pub(crate) fn prepare_updates_with_staging_for_scene_data_counts(
        &mut self,
        backend: &RenderBackend,
        encoder: &mut wgpu::CommandEncoder,
        scene_data_counts: [u32; 3],
    ) -> GpuScenePreparedUpload {
        if !GpuSceneStagingRing::should_stage(self.pending_upload_byte_len()) {
            return self.prepare_direct_updates_for_scene_data_counts(scene_data_counts);
        }

        self.prepare_staged_updates(backend, encoder, scene_data_counts)
    }

    pub(crate) fn prepare_updates_with_staging_for_virtual_geometry_counts(
        &mut self,
        backend: &RenderBackend,
        encoder: &mut wgpu::CommandEncoder,
        virtual_geometry_counts: [u32; 2],
    ) -> GpuScenePreparedUpload {
        let mut scene_data_counts = self.current_scene_data_counts();
        scene_data_counts[1] = virtual_geometry_counts[0];
        scene_data_counts[2] = virtual_geometry_counts[1];
        self.prepare_updates_with_staging_for_scene_data_counts(backend, encoder, scene_data_counts)
    }

    fn prepare_staged_updates(
        &mut self,
        backend: &RenderBackend,
        encoder: &mut wgpu::CommandEncoder,
        scene_data_counts: [u32; 3],
    ) -> GpuScenePreparedUpload {
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
        } else {
            let ranges = self
                .updates
                .prepare_primitive_upload_ranges(GPU_PRIMITIVE_DATA_STRIDE as u64)
                .to_vec();
            report.primitive_upload_range_count = ranges.len();
            for range in &ranges {
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
        } else {
            let ranges = self
                .updates
                .prepare_instance_upload_ranges(GPU_INSTANCE_DATA_STRIDE as u64)
                .to_vec();
            report.instance_upload_range_count = ranges.len();
            for range in &ranges {
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
        } else {
            let ranges = self
                .updates
                .prepare_light_upload_ranges(GpuLightData::STRIDE as u64)
                .to_vec();
            report.light_upload_range_count = ranges.len();
            for range in &ranges {
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

        let staging_upload = self.staging_ring.encode_upload(
            &backend.device,
            encoder,
            &self.primitive_buffer,
            &self.instance_buffer,
            &self.light_buffer,
        );
        let mut uploads = GpuSceneBufferUploadBatchBuilder::new();
        let uploaded_scene_data_counts =
            self.append_scene_data_count_param_uploads(&mut uploads, scene_data_counts);
        let mut batch = uploads.into_batch();
        if let Some(upload) = staging_upload {
            batch.push(upload);
        }
        GpuScenePreparedUpload::new(
            self,
            batch,
            uploaded_scene_data_counts,
            report,
            dirty_entry_count,
        )
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

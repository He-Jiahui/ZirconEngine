use crate::core::framework::render::GpuLightData;
use crate::graphics::backend::RenderBackend;
use crate::graphics::types::GraphicsError;
use zr_rhi_wgpu::WgpuBufferUploadBatch;

use super::binding::GpuSceneVisibleInstanceRemapParams;
use super::gpu_scene::{GpuScene, GpuSceneUploadReport};
use super::layout::{GPU_INSTANCE_DATA_STRIDE, GPU_PRIMITIVE_DATA_STRIDE};
use super::prepared_upload::GpuScenePreparedUpload;
use super::upload::GpuSceneBufferUploadBatchBuilder;

impl GpuScene {
    pub(crate) fn flush_updates(
        &mut self,
        backend: &RenderBackend,
    ) -> Result<GpuSceneUploadReport, GraphicsError> {
        let prepared = self.prepare_direct_updates();
        self.submit_prepared_upload(backend, prepared)
    }

    pub(super) fn prepare_direct_updates(&mut self) -> GpuScenePreparedUpload {
        let scene_data_counts = self.current_scene_data_counts();
        self.prepare_direct_updates_for_scene_data_counts(scene_data_counts)
    }

    pub(super) fn prepare_direct_updates_for_scene_data_counts(
        &mut self,
        scene_data_counts: [u32; 3],
    ) -> GpuScenePreparedUpload {
        let dirty_entry_count = self.updates.dirty_entry_count();
        let mut report = GpuSceneUploadReport::default();
        let mut uploads = GpuSceneBufferUploadBatchBuilder::new();

        if self.force_full_primitive_upload {
            let active_len = self.primitive_ids.high_water() as usize;
            let uploaded = uploads.push_pod_slice(
                &self.primitive_buffer,
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
            report.uploaded_bytes +=
                uploads.push_upload_ranges(&self.primitive_buffer, &self.primitive_shadow, &ranges);
        }

        if self.force_full_instance_upload {
            let active_len = self.instance_ids.high_water() as usize;
            let uploaded = uploads.push_pod_slice(
                &self.instance_buffer,
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
            report.uploaded_bytes +=
                uploads.push_upload_ranges(&self.instance_buffer, &self.instance_shadow, &ranges);
        }

        if self.force_full_light_upload {
            let uploaded = uploads.push_pod_slice(&self.light_buffer, 0, &self.light_shadow);
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
            report.uploaded_bytes +=
                uploads.push_upload_ranges(&self.light_buffer, &self.light_shadow, &ranges);
        }
        let uploaded_scene_data_counts =
            self.append_scene_data_count_param_uploads(&mut uploads, scene_data_counts);

        self.prepare_collected_updates(
            uploads,
            uploaded_scene_data_counts,
            report,
            dirty_entry_count,
        )
    }

    pub(super) fn prepare_collected_updates(
        &self,
        uploads: GpuSceneBufferUploadBatchBuilder,
        uploaded_scene_data_counts: Option<[u32; 3]>,
        report: GpuSceneUploadReport,
        dirty_entry_count: usize,
    ) -> GpuScenePreparedUpload {
        GpuScenePreparedUpload::new(
            self,
            uploads.into_batch(),
            uploaded_scene_data_counts,
            report,
            dirty_entry_count,
        )
    }

    pub(super) fn submit_prepared_upload(
        &mut self,
        backend: &RenderBackend,
        mut prepared: GpuScenePreparedUpload,
    ) -> Result<GpuSceneUploadReport, GraphicsError> {
        let mut batch = WgpuBufferUploadBatch::new();
        prepared.append_to(self, &mut batch);
        if !batch.is_empty() {
            backend.enqueue_copy_buffer_upload_batch(batch)?;
        }
        Ok(prepared.commit(self))
    }

    pub(super) fn commit_prepared_upload(
        &mut self,
        prepared: GpuScenePreparedUpload,
    ) -> GpuSceneUploadReport {
        assert!(
            std::sync::Arc::ptr_eq(&prepared.owner, &self.upload_transaction_owner),
            "GPU Scene commit target must own the prepared upload"
        );
        let GpuScenePreparedUpload {
            owner: _,
            batch: _,
            uploaded_scene_data_counts,
            report,
            dirty_entry_count,
            morph_commit,
            virtual_geometry_commit,
        } = prepared;
        self.updates.discard_primitive_updates();
        self.updates.discard_instance_updates();
        self.updates.discard_light_updates();
        self.force_full_primitive_upload = false;
        self.force_full_instance_upload = false;
        self.force_full_light_upload = false;
        self.primitive_ids.commit_pending_frees();
        self.instance_ids.commit_pending_frees();
        if let Some(counts) = uploaded_scene_data_counts {
            self.uploaded_scene_data_counts = Some(counts);
        }
        if let Some(commit) = morph_commit {
            commit.commit(self);
        }
        if let Some(commit) = virtual_geometry_commit {
            commit.commit(self);
        }
        self.refresh_stats(report.uploaded_bytes, dirty_entry_count);
        report
    }

    pub(super) fn current_scene_data_counts(&self) -> [u32; 3] {
        [
            u32::try_from(self.light_shadow.len()).expect("gpu scene light count exceeded u32"),
            u32::try_from(self.virtual_geometry_pages_shadow.len())
                .expect("gpu scene virtual geometry page count exceeded u32"),
            u32::try_from(self.virtual_geometry_clusters_shadow.len())
                .expect("gpu scene virtual geometry cluster count exceeded u32"),
        ]
    }

    pub(super) fn append_scene_data_count_param_uploads(
        &self,
        uploads: &mut GpuSceneBufferUploadBatchBuilder,
        counts: [u32; 3],
    ) -> Option<[u32; 3]> {
        if self.uploaded_scene_data_counts == Some(counts) {
            return None;
        }

        let direct = GpuSceneVisibleInstanceRemapParams::direct_with_scene_counts(
            counts[0], counts[1], counts[2],
        );
        uploads.push_pod_slice(
            &self.direct_visible_instance_remap_params_buffer,
            0,
            std::slice::from_ref(&direct),
        );
        let remapped = GpuSceneVisibleInstanceRemapParams::remapped_with_scene_counts(
            counts[0], counts[1], counts[2],
        );
        uploads.push_pod_slice(
            &self.remapped_visible_instance_remap_params_buffer,
            0,
            std::slice::from_ref(&remapped),
        );
        Some(counts)
    }
}

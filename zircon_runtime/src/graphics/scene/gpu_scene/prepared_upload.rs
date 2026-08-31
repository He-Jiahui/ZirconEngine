use crate::graphics::scene::gpu_scene::{GpuScene, GpuSceneUploadReport};
use std::sync::Arc;
use zr_rhi_wgpu::WgpuBufferUploadBatch;

use super::morph::{GpuSceneMorphUploadCommit, GpuScenePreparedMorphUpload};
use super::virtual_geometry::{
    GpuScenePreparedVirtualGeometryUpload, GpuSceneVirtualGeometryUploadCommit,
};

/// GPU Scene writes prepared during mesh extraction but not yet accepted by
/// the render backend. Dirty CPU state is committed only after the frame owner
/// has merged and accepted this batch.
pub(crate) struct GpuScenePreparedUpload {
    pub(super) owner: Arc<()>,
    pub(super) batch: WgpuBufferUploadBatch,
    pub(super) uploaded_scene_data_counts: Option<[u32; 3]>,
    pub(super) report: GpuSceneUploadReport,
    pub(super) dirty_entry_count: usize,
    pub(super) morph_commit: Option<GpuSceneMorphUploadCommit>,
    pub(super) virtual_geometry_commit: Option<GpuSceneVirtualGeometryUploadCommit>,
}

impl GpuScenePreparedUpload {
    pub(super) fn new(
        gpu_scene: &GpuScene,
        batch: WgpuBufferUploadBatch,
        uploaded_scene_data_counts: Option<[u32; 3]>,
        report: GpuSceneUploadReport,
        dirty_entry_count: usize,
    ) -> Self {
        Self {
            owner: Arc::clone(&gpu_scene.upload_transaction_owner),
            batch,
            uploaded_scene_data_counts,
            report,
            dirty_entry_count,
            morph_commit: None,
            virtual_geometry_commit: None,
        }
    }

    pub(crate) const fn report(&self) -> GpuSceneUploadReport {
        self.report
    }

    pub(crate) fn append_to(
        &mut self,
        gpu_scene: &GpuScene,
        frame_batch: &mut WgpuBufferUploadBatch,
    ) {
        self.assert_owned_by(gpu_scene);
        frame_batch.append(&mut self.batch);
    }

    pub(crate) fn append_additional_upload(
        &mut self,
        mut batch: WgpuBufferUploadBatch,
        uploaded_bytes: u64,
    ) {
        self.batch.append(&mut batch);
        self.report = self.report.with_additional_uploaded_bytes(uploaded_bytes);
    }

    pub(crate) fn append_morph_upload(&mut self, prepared: GpuScenePreparedMorphUpload) {
        assert!(
            prepared.is_owned_by(&self.owner),
            "GPU Scene frame cannot attach a foreign morph preparation"
        );
        let GpuScenePreparedMorphUpload {
            owner: _,
            mut batch,
            report,
            commit,
        } = prepared;
        assert!(
            self.morph_commit.is_none(),
            "GPU Scene frame must contain at most one morph upload"
        );
        self.morph_commit = Some(commit);
        self.batch.append(&mut batch);
        self.report = self
            .report
            .with_additional_uploaded_bytes(report.uploaded_bytes);
    }

    pub(crate) fn append_virtual_geometry_upload(
        &mut self,
        prepared: GpuScenePreparedVirtualGeometryUpload,
    ) {
        assert!(
            prepared.is_owned_by(&self.owner),
            "GPU Scene frame cannot attach a foreign virtual-geometry preparation"
        );
        let GpuScenePreparedVirtualGeometryUpload {
            owner: _,
            mut batch,
            report,
            commit,
        } = prepared;
        assert!(
            self.virtual_geometry_commit.is_none(),
            "GPU Scene frame must contain at most one virtual-geometry upload"
        );
        self.virtual_geometry_commit = Some(commit);
        self.batch.append(&mut batch);
        self.report = self
            .report
            .with_additional_uploaded_bytes(report.uploaded_bytes);
    }

    pub(crate) fn commit(self, gpu_scene: &mut GpuScene) -> GpuSceneUploadReport {
        self.assert_owned_by(gpu_scene);
        assert!(
            self.batch.is_empty(),
            "GPU Scene uploads must leave prepared ownership before dirty state is committed"
        );
        gpu_scene.commit_prepared_upload(self)
    }

    fn assert_owned_by(&self, gpu_scene: &GpuScene) {
        assert!(
            Arc::ptr_eq(&self.owner, &gpu_scene.upload_transaction_owner),
            "GPU Scene prepared upload cannot leave ownership for a foreign scene"
        );
    }
}

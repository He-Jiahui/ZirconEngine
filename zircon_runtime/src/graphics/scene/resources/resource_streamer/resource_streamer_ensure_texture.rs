use std::sync::Arc;

use crate::asset::TextureAsset;
use crate::core::framework::render::{
    RenderFrameSubmissionBoundaryReason, RenderFrameSubmissionProducer,
    RenderFrameSubmissionTransaction,
};
use crate::core::resource::{ResourceId, ResourceSnapshot};

use crate::graphics::backend::RenderBackend;
use crate::graphics::types::GraphicsError;

use super::super::prepared::PreparedTexture;
use super::super::{GpuTextureResource, GpuTextureUploadWork};
use super::ResourceStreamer;

pub(in crate::graphics::scene::resources) enum TextureSnapshotFramePrepareError {
    GpuArtifact,
    Submission(GraphicsError),
}

impl ResourceStreamer {
    pub(crate) fn ensure_texture(
        &mut self,
        backend: &RenderBackend,
        texture_layout: &wgpu::BindGroupLayout,
        id: ResourceId,
    ) -> Result<(), GraphicsError> {
        self.ensure_texture_internal(backend, texture_layout, id, None)
    }

    pub(crate) fn ensure_texture_for_frame(
        &mut self,
        backend: &RenderBackend,
        texture_layout: &wgpu::BindGroupLayout,
        id: ResourceId,
        submission_transaction: &mut RenderFrameSubmissionTransaction,
    ) -> Result<(), GraphicsError> {
        self.ensure_texture_internal(backend, texture_layout, id, Some(submission_transaction))
    }

    fn ensure_texture_internal(
        &mut self,
        backend: &RenderBackend,
        texture_layout: &wgpu::BindGroupLayout,
        id: ResourceId,
        submission_transaction: Option<&mut RenderFrameSubmissionTransaction>,
    ) -> Result<(), GraphicsError> {
        let requested_revision = self.resource_revision(id)?;
        if self
            .textures
            .get(&id)
            .is_some_and(|prepared| prepared.revision == requested_revision)
        {
            return Ok(());
        }
        let texture = self
            .asset_manager()?
            .load_texture_asset_snapshot(id)
            .map_err(|error| GraphicsError::Asset(error.to_string()))?;
        self.ensure_texture_snapshot_internal(
            backend,
            texture_layout,
            id,
            texture,
            submission_transaction,
        )
    }

    pub(in crate::graphics::scene::resources) fn ensure_texture_snapshot_for_frame(
        &mut self,
        backend: &RenderBackend,
        texture_layout: &wgpu::BindGroupLayout,
        id: ResourceId,
        texture: ResourceSnapshot<TextureAsset>,
        submission_transaction: &mut RenderFrameSubmissionTransaction,
    ) -> Result<(), TextureSnapshotFramePrepareError> {
        if self
            .textures
            .get(&id)
            .is_some_and(|prepared| prepared.revision == texture.revision())
        {
            return Ok(());
        }
        let (revision, capture_sample_rgba, work) = self
            .prepare_texture_snapshot_upload_work(backend, texture_layout, id, texture)
            .map_err(|_| TextureSnapshotFramePrepareError::GpuArtifact)?;
        let resource = self
            .enqueue_gpu_texture_upload_work_internal(
                backend,
                work,
                Some((id, submission_transaction)),
            )
            .map_err(TextureSnapshotFramePrepareError::Submission)?;
        self.publish_prepared_texture(id, revision, resource, capture_sample_rgba);
        Ok(())
    }

    fn ensure_texture_snapshot_internal(
        &mut self,
        backend: &RenderBackend,
        texture_layout: &wgpu::BindGroupLayout,
        id: ResourceId,
        texture: ResourceSnapshot<TextureAsset>,
        submission_transaction: Option<&mut RenderFrameSubmissionTransaction>,
    ) -> Result<(), GraphicsError> {
        let revision = texture.revision();
        if self
            .textures
            .get(&id)
            .is_some_and(|prepared| prepared.revision == revision)
        {
            return Ok(());
        }
        let (revision, capture_sample_rgba, work) =
            self.prepare_texture_snapshot_upload_work(backend, texture_layout, id, texture)?;
        let resource = self.enqueue_gpu_texture_upload_work_internal(
            backend,
            work,
            submission_transaction.map(|transaction| (id, transaction)),
        )?;
        self.publish_prepared_texture(id, revision, resource, capture_sample_rgba);
        Ok(())
    }

    fn prepare_texture_snapshot_upload_work(
        &self,
        backend: &RenderBackend,
        texture_layout: &wgpu::BindGroupLayout,
        id: ResourceId,
        texture: ResourceSnapshot<TextureAsset>,
    ) -> Result<(u64, Option<[f32; 4]>, GpuTextureUploadWork), GraphicsError> {
        let revision = texture.revision();
        let capture_sample_rgba = center_texture_asset_rgba(&texture);
        let work = GpuTextureResource::from_asset(
            &backend.device,
            texture_layout,
            Arc::clone(&self.texture_sampler_cache),
            id,
            (*texture).clone(),
            &self.runtime_mip_gen_pass,
        )?;
        Ok((revision, capture_sample_rgba, work))
    }

    fn publish_prepared_texture(
        &mut self,
        id: ResourceId,
        revision: u64,
        resource: Arc<GpuTextureResource>,
        capture_sample_rgba: Option<[f32; 4]>,
    ) {
        self.textures.insert(
            id,
            PreparedTexture::fully_resident(revision, resource, capture_sample_rgba),
        );
        self.mip_streaming_states.remove(&id);
    }

    pub(crate) fn ensure_sprite_texture(
        &mut self,
        backend: &RenderBackend,
        texture_layout: &wgpu::BindGroupLayout,
        id: ResourceId,
    ) -> Result<(), GraphicsError> {
        self.ensure_texture(backend, texture_layout, id)
    }

    pub(crate) fn ensure_sprite_texture_for_frame(
        &mut self,
        backend: &RenderBackend,
        texture_layout: &wgpu::BindGroupLayout,
        id: ResourceId,
        submission_transaction: &mut RenderFrameSubmissionTransaction,
    ) -> Result<(), GraphicsError> {
        self.ensure_texture_for_frame(backend, texture_layout, id, submission_transaction)
    }

    pub(super) fn enqueue_gpu_texture_upload_work(
        &self,
        backend: &RenderBackend,
        work: GpuTextureUploadWork,
    ) -> Result<Arc<GpuTextureResource>, GraphicsError> {
        self.enqueue_gpu_texture_upload_work_internal(backend, work, None)
    }

    pub(super) fn enqueue_gpu_texture_upload_work_for_frame(
        &self,
        backend: &RenderBackend,
        resource_id: ResourceId,
        work: GpuTextureUploadWork,
        submission_transaction: &mut RenderFrameSubmissionTransaction,
    ) -> Result<Arc<GpuTextureResource>, GraphicsError> {
        self.enqueue_gpu_texture_upload_work_internal(
            backend,
            work,
            Some((resource_id, submission_transaction)),
        )
    }

    fn enqueue_gpu_texture_upload_work_internal(
        &self,
        backend: &RenderBackend,
        work: GpuTextureUploadWork,
        mut frame_submission: Option<(ResourceId, &mut RenderFrameSubmissionTransaction)>,
    ) -> Result<Arc<GpuTextureResource>, GraphicsError> {
        if !work.pre_upload_commands.is_empty() {
            let ticket = backend.submit_graphics_command_buffers(work.pre_upload_commands)?;
            if let Some((resource_id, transaction)) = frame_submission.as_mut() {
                backend.record_pre_scene_resource_submission_with_boundary(
                    transaction,
                    RenderFrameSubmissionProducer::TexturePreUpload,
                    *resource_id,
                    RenderFrameSubmissionBoundaryReason::TextureMipPreservationBeforeUpload,
                    ticket,
                )?;
            }
        }
        let ticket = backend.enqueue_copy_texture_upload_batch(work.upload_batch)?;
        if let Some((resource_id, transaction)) = frame_submission.as_mut() {
            backend.record_pre_scene_resource_submission(
                transaction,
                RenderFrameSubmissionProducer::TextureCopyUpload,
                *resource_id,
                ticket,
            )?;
        }
        if !work.post_upload_commands.is_empty() {
            let ticket = backend.enqueue_graphics_command_buffers(work.post_upload_commands)?;
            if let Some((resource_id, transaction)) = frame_submission.as_mut() {
                backend.record_pre_scene_resource_submission(
                    transaction,
                    RenderFrameSubmissionProducer::TexturePostUpload,
                    *resource_id,
                    ticket,
                )?;
            }
        }
        Ok(Arc::new(work.resource))
    }
}

fn center_texture_asset_rgba(texture: &TextureAsset) -> Option<[f32; 4]> {
    if texture.width == 0 || texture.height == 0 {
        return None;
    }

    let x = (texture.width / 2) as usize;
    let y = (texture.height / 2) as usize;
    let index = ((y * texture.width as usize) + x) * 4;
    let rgba = texture.rgba.get(index..index + 4)?;
    Some([
        rgba[0] as f32 / 255.0,
        rgba[1] as f32 / 255.0,
        rgba[2] as f32 / 255.0,
        rgba[3] as f32 / 255.0,
    ])
}

#[cfg(test)]
mod tests {
    use crate::asset::{AssetUri, TextureAsset};

    use super::center_texture_asset_rgba;

    #[test]
    fn capture_sample_reads_the_center_texel_once_from_rgba8_source() {
        let texture = TextureAsset::new_rgba8(
            AssetUri::parse("res://textures/capture-center.png").expect("texture uri"),
            2,
            2,
            vec![
                255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 64, 128, 192, 255,
            ],
        );

        assert_eq!(
            center_texture_asset_rgba(&texture),
            Some([64.0 / 255.0, 128.0 / 255.0, 192.0 / 255.0, 1.0])
        );
    }

    #[test]
    fn gpu_texture_publication_uses_one_atomic_asset_revision_snapshot() {
        let production = include_str!("resource_streamer_ensure_texture.rs")
            .split_once("#[cfg(test)]")
            .map(|(production, _)| production)
            .expect("texture preparation test boundary");

        assert!(production.contains("load_texture_asset_snapshot(id)"));
        assert!(production.contains("let revision = texture.revision();"));
        assert!(!production.contains("let revision = self.resource_revision(id)?;"));
    }

    #[test]
    fn frame_texture_upload_records_pre_copy_post_ticket_order() {
        let production = include_str!("resource_streamer_ensure_texture.rs")
            .split_once("#[cfg(test)]")
            .map(|(production, _)| production)
            .expect("texture preparation test boundary");
        let pre = production
            .find("RenderFrameSubmissionProducer::TexturePreUpload")
            .expect("pre-upload producer");
        let copy = production
            .find("RenderFrameSubmissionProducer::TextureCopyUpload")
            .expect("copy-upload producer");
        let post = production
            .find("RenderFrameSubmissionProducer::TexturePostUpload")
            .expect("post-upload producer");

        assert!(pre < copy && copy < post);
        assert_eq!(production.matches("let ticket = backend.").count(), 3);
        assert_eq!(
            production
                .matches("record_pre_scene_resource_submission(")
                .count(),
            2
        );
        assert_eq!(
            production
                .matches("record_pre_scene_resource_submission_with_boundary(")
                .count(),
            1
        );
        assert!(
            production.contains(
                "RenderFrameSubmissionBoundaryReason::TextureMipPreservationBeforeUpload"
            )
        );
    }
}

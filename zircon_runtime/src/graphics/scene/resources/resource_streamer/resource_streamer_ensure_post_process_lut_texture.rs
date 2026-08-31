use std::sync::Arc;

use crate::asset::TextureAsset;
use crate::core::framework::render::{
    RenderFrameSubmissionProducer, RenderFrameSubmissionTransaction,
};
use crate::core::resource::{ResourceId, ResourceSnapshot};
use crate::graphics::backend::RenderBackend;
use crate::graphics::types::GraphicsError;

use super::super::prepared::PreparedPostProcessLutTexture;
use super::super::{PostProcessLutTextureResource, PostProcessLutTextureUploadWork};
use super::ResourceStreamer;

impl ResourceStreamer {
    pub(crate) fn ensure_post_process_lut_texture(
        &mut self,
        backend: &RenderBackend,
        id: ResourceId,
        submission_transaction: &mut RenderFrameSubmissionTransaction,
    ) -> Result<(), GraphicsError> {
        let requested_revision = self.resource_revision(id)?;
        if self
            .post_process_lut_textures
            .get(&id)
            .is_some_and(|prepared| prepared.revision == requested_revision)
        {
            return Ok(());
        }

        let texture = self
            .asset_manager()?
            .load_texture_asset_snapshot(id)
            .map_err(|error| GraphicsError::Asset(error.to_string()))?;
        self.publish_post_process_lut_texture_snapshot(backend, id, texture, submission_transaction)
    }

    pub(super) fn ensure_post_process_lut_texture_snapshot(
        &mut self,
        backend: &RenderBackend,
        id: ResourceId,
        texture: ResourceSnapshot<TextureAsset>,
        submission_transaction: &mut RenderFrameSubmissionTransaction,
    ) -> Result<(), GraphicsError> {
        if self
            .post_process_lut_textures
            .get(&id)
            .is_some_and(|prepared| prepared.revision == texture.revision())
        {
            return Ok(());
        }
        self.publish_post_process_lut_texture_snapshot(backend, id, texture, submission_transaction)
    }

    fn publish_post_process_lut_texture_snapshot(
        &mut self,
        backend: &RenderBackend,
        id: ResourceId,
        texture: ResourceSnapshot<TextureAsset>,
        submission_transaction: &mut RenderFrameSubmissionTransaction,
    ) -> Result<(), GraphicsError> {
        let revision = texture.revision();
        let PostProcessLutTextureUploadWork {
            resource,
            upload_batch,
        } = PostProcessLutTextureResource::prepare_from_rgba8_asset(&backend.device, id, &texture)?;
        let ticket = backend.enqueue_copy_texture_upload_batch(upload_batch)?;
        backend.record_pre_scene_resource_submission(
            submission_transaction,
            RenderFrameSubmissionProducer::TextureCopyUpload,
            id,
            ticket,
        )?;
        self.post_process_lut_textures.insert(
            id,
            PreparedPostProcessLutTexture {
                revision,
                resource: Arc::new(resource),
            },
        );
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn lut_publication_uses_one_snapshot_and_the_frame_texture_transaction() {
        let production = include_str!("resource_streamer_ensure_post_process_lut_texture.rs")
            .split_once("#[cfg(test)]")
            .map(|(production, _)| production)
            .expect("LUT streamer test boundary");
        let prepare = production
            .find("prepare_from_rgba8_asset(")
            .expect("LUT preparation");
        let enqueue = production
            .find("enqueue_copy_texture_upload_batch(upload_batch)")
            .expect("backend upload admission");
        let record = production
            .find("record_pre_scene_resource_submission(")
            .expect("frame transaction record");
        let publish = production
            .find("PreparedPostProcessLutTexture {")
            .expect("same-frame LUT publication");

        assert!(production.contains("load_texture_asset_snapshot(id)"));
        assert!(production.contains("let revision = texture.revision();"));
        assert!(!production.contains("(*texture).clone()"));
        assert!(production.contains("ensure_post_process_lut_texture_snapshot("));
        assert!(prepare < enqueue && enqueue < record && record < publish);
        assert!(!production.contains("wgpu::Queue"));
        assert!(!production.contains("queue.write_texture"));
    }
}

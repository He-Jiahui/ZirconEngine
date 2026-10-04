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

    /// 先登记后端纹理上传与帧提交事务，再把同一修订快照暴露给后续后处理读取。
    /// 这保证同帧的消费者不会先看到尚未纳入提交序列的资源。
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
#[path = "tests/resource_streamer_ensure_post_process_lut_texture.rs"]
mod tests;

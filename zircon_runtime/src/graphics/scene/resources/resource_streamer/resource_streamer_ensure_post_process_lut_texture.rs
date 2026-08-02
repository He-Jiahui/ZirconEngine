use std::sync::Arc;

use crate::core::resource::ResourceId;
use crate::graphics::types::GraphicsError;

use super::super::prepared::PreparedPostProcessLutTexture;
use super::super::PostProcessLutTextureResource;
use super::ResourceStreamer;

impl ResourceStreamer {
    pub(crate) fn ensure_post_process_lut_texture(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        id: ResourceId,
    ) -> Result<(), GraphicsError> {
        let revision = self.resource_revision(id)?;
        if self
            .post_process_lut_textures
            .get(&id)
            .is_some_and(|prepared| prepared.revision == revision)
        {
            return Ok(());
        }

        let texture = self
            .asset_manager()?
            .load_texture_asset(id)
            .map_err(|error| GraphicsError::Asset(error.to_string()))?;
        let resource = Arc::new(PostProcessLutTextureResource::from_rgba8_asset(
            device, queue, id, texture,
        )?);
        self.post_process_lut_textures
            .insert(id, PreparedPostProcessLutTexture { revision, resource });
        Ok(())
    }
}

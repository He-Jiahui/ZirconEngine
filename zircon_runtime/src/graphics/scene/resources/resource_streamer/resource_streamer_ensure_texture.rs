use std::sync::Arc;

use crate::core::resource::ResourceId;

use crate::graphics::types::GraphicsError;

use super::super::prepared::PreparedTexture;
use super::super::GpuTextureResource;
use super::ResourceStreamer;

impl ResourceStreamer {
    pub(crate) fn ensure_texture(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        texture_layout: &wgpu::BindGroupLayout,
        id: ResourceId,
    ) -> Result<(), GraphicsError> {
        let revision = self.resource_revision(id)?;
        if self
            .textures
            .get(&id)
            .is_some_and(|prepared| prepared.revision == revision)
        {
            return Ok(());
        }
        let texture = self
            .asset_manager
            .load_texture_asset(id)
            .map_err(|error| GraphicsError::Asset(error.to_string()))?;
        let resource = Arc::new(GpuTextureResource::from_asset(
            device,
            queue,
            texture_layout,
            id,
            texture,
        ));
        self.textures
            .insert(id, PreparedTexture { revision, resource });
        Ok(())
    }
}

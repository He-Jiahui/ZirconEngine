use std::sync::Arc;

use crate::core::resource::ResourceId;
use crate::graphics::types::{GraphicsError, ViewportRenderFrame};

use super::super::prepared::PreparedOutputTargetTexture;
use super::super::OutputTargetTextureResource;
use super::ResourceStreamer;

impl ResourceStreamer {
    pub(super) fn ensure_output_target_texture(
        &mut self,
        device: &wgpu::Device,
        frame: &ViewportRenderFrame,
    ) -> Result<(), GraphicsError> {
        let Some(texture) = output_target_texture_id(frame) else {
            return Ok(());
        };
        self.ensure_output_target_texture_resource(device, texture)
    }

    pub(super) fn ensure_output_target_texture_resource(
        &mut self,
        device: &wgpu::Device,
        id: ResourceId,
    ) -> Result<(), GraphicsError> {
        let requested_revision = self.resource_revision(id)?;
        if self
            .output_target_textures
            .get(&id)
            .is_some_and(|prepared| prepared.revision == requested_revision)
        {
            return Ok(());
        }

        let texture = self
            .asset_manager()?
            .load_texture_asset_snapshot(id)
            .map_err(|error| GraphicsError::Asset(error.to_string()))?;
        let revision = texture.revision();
        let resource = Arc::new(OutputTargetTextureResource::from_asset(
            device,
            id,
            (*texture).clone(),
        )?);
        self.output_target_textures
            .insert(id, PreparedOutputTargetTexture { revision, resource });
        Ok(())
    }
}

fn output_target_texture_id(frame: &ViewportRenderFrame) -> Option<ResourceId> {
    frame
        .output_target()
        .texture_handle()
        .map(|texture| texture.id())
}

#[cfg(test)]
#[path = "tests/resource_streamer_ensure_output_target_texture.rs"]
mod tests;

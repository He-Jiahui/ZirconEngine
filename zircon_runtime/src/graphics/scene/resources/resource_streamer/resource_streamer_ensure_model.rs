use std::sync::Arc;

use crate::asset::ModelAsset;
use crate::core::resource::{ModelMarker, ResourceHandle};

use crate::graphics::types::GraphicsError;

use super::super::prepared::PreparedModel;
use super::super::GpuModelResource;
use super::ResourceStreamer;

impl ResourceStreamer {
    pub(crate) fn ensure_model(
        &mut self,
        device: &wgpu::Device,
        handle: ResourceHandle<ModelMarker>,
    ) -> Result<(), GraphicsError> {
        let id = handle.id();
        let revision = self.resource_revision(id)?;
        if self
            .models
            .get(&id)
            .is_some_and(|prepared| prepared.revision == revision)
        {
            return Ok(());
        }
        let model = self
            .asset_manager
            .load_model_asset(id)
            .map_err(|error| GraphicsError::Asset(error.to_string()))?;
        let asset = Arc::<ModelAsset>::new(model);
        let resource = Arc::new(GpuModelResource::from_asset(device, id, asset.as_ref()));
        self.models.insert(
            id,
            PreparedModel {
                revision,
                asset,
                resource,
            },
        );
        Ok(())
    }
}

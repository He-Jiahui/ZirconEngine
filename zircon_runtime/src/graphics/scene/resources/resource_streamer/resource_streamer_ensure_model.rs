use std::sync::Arc;

use crate::asset::{AssetReference, MeshAsset, ModelAsset};
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
        let asset_manager = self.asset_manager.clone();
        let resource = Arc::new(GpuModelResource::from_asset_with_mesh_assets(
            device,
            id,
            asset.as_ref(),
            |reference| load_referenced_mesh_asset(asset_manager.as_ref(), reference),
        ));
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

fn load_referenced_mesh_asset(
    asset_manager: &crate::asset::pipeline::manager::ProjectAssetManager,
    reference: &AssetReference,
) -> Option<MeshAsset> {
    let id = asset_manager
        .resource_manager()
        .registry()
        .get_by_locator(&reference.locator)
        .map(|record| record.id())?;
    asset_manager.load_mesh_asset(id).ok()
}

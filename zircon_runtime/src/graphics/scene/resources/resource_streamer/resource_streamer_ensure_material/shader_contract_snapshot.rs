use crate::asset::{AssetReference, ShaderAsset};
use crate::core::resource::{ResourceId, ResourceSnapshot};

use super::super::ResourceStreamer;

pub(in crate::graphics::scene::resources::resource_streamer) struct MaterialShaderContractSnapshot {
    resource_id: ResourceId,
    asset: ResourceSnapshot<ShaderAsset>,
}

impl MaterialShaderContractSnapshot {
    pub(in crate::graphics::scene::resources::resource_streamer) const fn resource_id(
        &self,
    ) -> ResourceId {
        self.resource_id
    }

    pub(in crate::graphics::scene::resources::resource_streamer) fn revision(&self) -> u64 {
        self.asset.revision()
    }

    pub(in crate::graphics::scene::resources::resource_streamer) fn asset(&self) -> &ShaderAsset {
        &self.asset
    }
}

impl ResourceStreamer {
    pub(in crate::graphics::scene::resources::resource_streamer) fn load_shader_contract(
        asset_manager: &crate::asset::ProjectAssetManager,
        reference: AssetReference,
    ) -> Option<MaterialShaderContractSnapshot> {
        let resource_id = asset_manager.resolve_asset_id(&reference.locator)?;
        let asset = asset_manager.load_shader_asset_snapshot(resource_id).ok()?;
        Some(MaterialShaderContractSnapshot { resource_id, asset })
    }
}

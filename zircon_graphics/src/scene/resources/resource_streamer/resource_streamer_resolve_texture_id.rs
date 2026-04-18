use zircon_asset::MaterialAsset;
use zircon_resource::ResourceId;

use super::ResourceStreamer;

impl ResourceStreamer {
    pub(crate) fn resolve_texture_id(&self, material: &MaterialAsset) -> Option<ResourceId> {
        material.base_color_texture.as_ref().and_then(|reference| {
            self.asset_manager
                .resource_manager()
                .registry()
                .get_by_locator(&reference.locator)
                .map(|record| record.id())
        })
    }
}

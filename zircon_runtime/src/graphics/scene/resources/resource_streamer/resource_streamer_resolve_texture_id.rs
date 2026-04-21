use crate::asset::AssetReference;
use crate::core::resource::ResourceId;

use super::ResourceStreamer;

impl ResourceStreamer {
    pub(crate) fn resolve_texture_id(
        &self,
        reference: Option<&AssetReference>,
    ) -> Option<ResourceId> {
        reference.and_then(|reference| {
            self.asset_manager
                .resource_manager()
                .registry()
                .get_by_locator(&reference.locator)
                .map(|record| record.id())
        })
    }
}

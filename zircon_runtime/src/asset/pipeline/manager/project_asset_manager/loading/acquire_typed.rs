use crate::core::CoreError;
use crate::core::resource::{ResourceData, ResourceHandle, ResourceLease, ResourceMarker};

use super::super::super::errors::asset_error_message;
use super::super::ProjectAssetManager;
use crate::asset::AssetId;

impl ProjectAssetManager {
    pub(in crate::asset::pipeline::manager::project_asset_manager::loading) fn acquire_typed<
        TMarker,
        TAsset,
    >(
        &self,
        id: AssetId,
        handle: ResourceHandle<TMarker>,
        label: &str,
    ) -> Result<ResourceLease<TAsset>, CoreError>
    where
        TMarker: ResourceMarker,
        TAsset: ResourceData,
    {
        self.ensure_resident(id)?;
        self.resource_manager()
            .acquire::<TMarker, TAsset>(handle)
            .ok_or_else(|| asset_error_message(format!("asset {id} was not a ready {label}")))
    }
}

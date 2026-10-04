use crate::core::resource::{ResourceData, ResourceHandle, ResourceMarker, ResourceSnapshot};
use crate::core::CoreError;

use super::super::super::errors::asset_error_message;
use super::super::ProjectAssetManager;
use crate::asset::AssetId;

impl ProjectAssetManager {
    /// 供渲染等跨帧消费者同时固定载荷和修订版，避免热重载后值与版本分别来自两次查询。
    pub(in crate::asset::pipeline::manager::project_asset_manager::loading) fn load_typed_snapshot<
        TMarker,
        TAsset,
    >(
        &self,
        id: AssetId,
        handle: ResourceHandle<TMarker>,
        label: &str,
    ) -> Result<ResourceSnapshot<TAsset>, CoreError>
    where
        TMarker: ResourceMarker,
        TAsset: ResourceData,
    {
        self.ensure_resident(id)?;
        self.resource_manager()
            .snapshot::<TMarker, TAsset>(handle)
            .ok_or_else(|| asset_error_message(format!("asset {id} was not a ready {label}")))
    }
}

#[cfg(test)]
#[path = "tests/load_typed_snapshot.rs"]
mod tests;

use crate::core::resource::{ResourceData, ResourceHandle, ResourceLease, ResourceMarker};
use crate::core::CoreError;

use super::super::super::errors::asset_error_message;
use super::super::ProjectAssetManager;
use crate::asset::AssetId;

impl ProjectAssetManager {
    /// 供各类 acquire_* 入口复用：先确保驻留，再交出能跨资源替换持有该载荷的 lease。
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

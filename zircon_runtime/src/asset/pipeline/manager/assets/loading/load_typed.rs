use crate::core::resource::{ResourceData, ResourceHandle, ResourceMarker};
use crate::core::CoreError;

use super::super::super::errors::asset_error_message;
use super::super::ProjectAssetManager;
use crate::asset::AssetId;

impl ProjectAssetManager {
    /// 供 load_* 入口取得当前资产值的独立副本；需要精确修订版身份时应取 snapshot。
    pub(in crate::asset::pipeline::manager::project_asset_manager::loading) fn load_typed<
        TMarker,
        TAsset,
    >(
        &self,
        id: AssetId,
        handle: ResourceHandle<TMarker>,
        label: &str,
    ) -> Result<TAsset, CoreError>
    where
        TMarker: ResourceMarker,
        TAsset: ResourceData + Clone,
    {
        self.ensure_resident(id)?;
        self.resource_manager()
            .get::<TMarker, TAsset>(handle)
            .map(|asset| asset.as_ref().clone())
            .ok_or_else(|| asset_error_message(format!("asset {id} was not a ready {label}")))
    }
}

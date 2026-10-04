use std::sync::Arc;

use crate::asset::ModelAsset;
use crate::core::resource::ResourceId;

use super::ResourceStreamer;

impl ResourceStreamer {
    /// 模型源修订未变时复用 PreparedModel 持有的 Arc；复合几何修订只驱动 GPU 几何失效，
    /// 不应强迫每个实例重新加载源资产。
    pub(crate) fn load_model_asset(&self, id: ResourceId) -> Option<Arc<ModelAsset>> {
        let asset_manager = self.asset_manager().ok()?;
        load_model_asset_with_cache(
            self.models
                .get(&id)
                .map(|prepared| (&prepared.asset, prepared.source_revision)),
            self.resource_revision(id).ok(),
            || asset_manager.load_model_asset(id).ok().map(Arc::new),
        )
    }
}

fn load_model_asset_with_cache<F>(
    prepared: Option<(&Arc<ModelAsset>, u64)>,
    current_revision: Option<u64>,
    fallback_load: F,
) -> Option<Arc<ModelAsset>>
where
    F: FnOnce() -> Option<Arc<ModelAsset>>,
{
    if let (Some((asset, prepared_revision)), Some(current_revision)) = (prepared, current_revision)
    {
        if prepared_revision == current_revision {
            return Some(Arc::clone(asset));
        }
    }
    fallback_load()
}

#[cfg(test)]
#[path = "tests/resource_streamer_load_model_asset.rs"]
mod tests;

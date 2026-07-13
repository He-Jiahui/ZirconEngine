use crate::asset::{AssetReference, SceneAsset};

pub(super) fn extract(asset: &SceneAsset) -> Vec<AssetReference> {
    asset.direct_references()
}

use crate::asset::{AssetReference, ModelAsset};

pub(super) fn extract(asset: &ModelAsset) -> Vec<AssetReference> {
    asset.direct_references()
}

use crate::asset::{AssetReference, MaterialAsset};

pub(super) fn extract(asset: &MaterialAsset) -> Vec<AssetReference> {
    asset.direct_references()
}

use crate::asset::{asset_kind_for_imported_asset, AssetKind, ImportedAsset};

pub(super) fn asset_kind(imported: &ImportedAsset) -> AssetKind {
    asset_kind_for_imported_asset(imported)
}

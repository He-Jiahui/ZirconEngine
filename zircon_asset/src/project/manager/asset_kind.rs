use crate::{AssetKind, ImportedAsset};

pub(super) fn asset_kind(imported: &ImportedAsset) -> AssetKind {
    match imported {
        ImportedAsset::Texture(_) => AssetKind::Texture,
        ImportedAsset::Shader(_) => AssetKind::Shader,
        ImportedAsset::Material(_) => AssetKind::Material,
        ImportedAsset::Scene(_) => AssetKind::Scene,
        ImportedAsset::Model(_) => AssetKind::Model,
        ImportedAsset::UiLayout(_) => AssetKind::UiLayout,
        ImportedAsset::UiWidget(_) => AssetKind::UiWidget,
        ImportedAsset::UiStyle(_) => AssetKind::UiStyle,
    }
}

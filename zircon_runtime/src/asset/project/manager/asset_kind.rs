use crate::asset::{AssetKind, ImportedAsset};

pub(super) fn asset_kind(imported: &ImportedAsset) -> AssetKind {
    match imported {
        ImportedAsset::Texture(_) => AssetKind::Texture,
        ImportedAsset::Shader(_) => AssetKind::Shader,
        ImportedAsset::Material(_) => AssetKind::Material,
        ImportedAsset::Sound(_) => AssetKind::Sound,
        ImportedAsset::Font(_) => AssetKind::Font,
        ImportedAsset::Scene(_) => AssetKind::Scene,
        ImportedAsset::Model(_) => AssetKind::Model,
        ImportedAsset::UiLayout(_) => AssetKind::UiLayout,
        ImportedAsset::UiWidget(_) => AssetKind::UiWidget,
        ImportedAsset::UiStyle(_) => AssetKind::UiStyle,
        ImportedAsset::PhysicsMaterial(_) => AssetKind::PhysicsMaterial,
        ImportedAsset::AnimationSkeleton(_) => AssetKind::AnimationSkeleton,
        ImportedAsset::AnimationClip(_) => AssetKind::AnimationClip,
        ImportedAsset::AnimationSequence(_) => AssetKind::AnimationSequence,
        ImportedAsset::AnimationGraph(_) => AssetKind::AnimationGraph,
        ImportedAsset::AnimationStateMachine(_) => AssetKind::AnimationStateMachine,
    }
}

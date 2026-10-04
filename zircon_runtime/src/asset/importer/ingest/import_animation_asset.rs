use crate::asset::assets::ImportedAsset;
use crate::asset::{AssetImportContext, AssetImportError, AssetImportOutcome};
use crate::core::framework::animation::{
    AnimationClipAsset, AnimationGraphAsset, AnimationSequenceAsset, AnimationSkeletonAsset,
    AnimationStateMachineAsset,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum AnimationAssetKind {
    Skeleton,
    Clip,
    Sequence,
    Graph,
    StateMachine,
}

impl AnimationAssetKind {
    fn from_file_name(file_name: &str) -> Option<Self> {
        const SUFFIXES: [(&str, AnimationAssetKind); 5] = [
            (".skeleton.zranim", AnimationAssetKind::Skeleton),
            (".clip.zranim", AnimationAssetKind::Clip),
            (".sequence.zranim", AnimationAssetKind::Sequence),
            (".graph.zranim", AnimationAssetKind::Graph),
            (".state_machine.zranim", AnimationAssetKind::StateMachine),
        ];
        SUFFIXES.iter().find_map(|(suffix, kind)| {
            file_name
                .get(file_name.len().checked_sub(suffix.len())?..)
                .is_some_and(|tail| tail.eq_ignore_ascii_case(suffix))
                .then_some(*kind)
        })
    }
}

// registry 的五个 .zranim 描述符共享此入口；文件全后缀选定骨架、clip、序列、图或状态机类型，
// 类型与源 URI 一起交给 AssetImportOutcome，调用者须保持路径后缀与登记描述符一致。
pub(crate) fn import_animation_asset(
    context: &AssetImportContext,
) -> Result<AssetImportOutcome, AssetImportError> {
    let file_name = context
        .source_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default();

    match AnimationAssetKind::from_file_name(file_name) {
        Some(AnimationAssetKind::Skeleton) => {
            AnimationSkeletonAsset::from_bytes(&context.source_bytes)
                .map(ImportedAsset::AnimationSkeleton)
                .map(|asset| AssetImportOutcome::new(context.uri.clone(), asset))
                .map_err(AssetImportError::AnimationAsset)
        }
        Some(AnimationAssetKind::Clip) => AnimationClipAsset::from_bytes(&context.source_bytes)
            .map(ImportedAsset::AnimationClip)
            .map(|asset| AssetImportOutcome::new(context.uri.clone(), asset))
            .map_err(AssetImportError::AnimationAsset),
        Some(AnimationAssetKind::Sequence) => {
            AnimationSequenceAsset::from_bytes(&context.source_bytes)
                .map(ImportedAsset::AnimationSequence)
                .map(|asset| AssetImportOutcome::new(context.uri.clone(), asset))
                .map_err(AssetImportError::AnimationAsset)
        }
        Some(AnimationAssetKind::Graph) => AnimationGraphAsset::from_bytes(&context.source_bytes)
            .map(ImportedAsset::AnimationGraph)
            .map(|asset| AssetImportOutcome::new(context.uri.clone(), asset))
            .map_err(AssetImportError::AnimationAsset),
        Some(AnimationAssetKind::StateMachine) => {
            AnimationStateMachineAsset::from_bytes(&context.source_bytes)
                .map(ImportedAsset::AnimationStateMachine)
                .map(|asset| AssetImportOutcome::new(context.uri.clone(), asset))
                .map_err(AssetImportError::AnimationAsset)
        }
        None => Err(AssetImportError::UnsupportedFormat(format!(
            "unknown animation asset suffix for {}",
            context.source_path.display()
        ))),
    }
}

#[cfg(test)]
#[path = "tests/import_animation_asset_plugins07_animation_hotpath_tests.rs"]
mod plugins07_animation_hotpath_tests;

use crate::asset::assets::ImportedAsset;
use crate::asset::{
    AnimationClipAsset, AnimationGraphAsset, AnimationSequenceAsset, AnimationSkeletonAsset,
    AnimationStateMachineAsset, AssetImportContext, AssetImportError, AssetImportOutcome,
};

pub(crate) fn import_animation_asset(
    context: &AssetImportContext,
) -> Result<AssetImportOutcome, AssetImportError> {
    let lower_name = context
        .source_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();

    if lower_name.ends_with(".skeleton.zranim") {
        return AnimationSkeletonAsset::from_bytes(&context.source_bytes)
            .map(ImportedAsset::AnimationSkeleton)
            .map(AssetImportOutcome::new)
            .map_err(AssetImportError::Parse);
    }
    if lower_name.ends_with(".clip.zranim") {
        return AnimationClipAsset::from_bytes(&context.source_bytes)
            .map(ImportedAsset::AnimationClip)
            .map(AssetImportOutcome::new)
            .map_err(AssetImportError::Parse);
    }
    if lower_name.ends_with(".sequence.zranim") {
        return AnimationSequenceAsset::from_bytes(&context.source_bytes)
            .map(ImportedAsset::AnimationSequence)
            .map(AssetImportOutcome::new)
            .map_err(AssetImportError::Parse);
    }
    if lower_name.ends_with(".graph.zranim") {
        return AnimationGraphAsset::from_bytes(&context.source_bytes)
            .map(ImportedAsset::AnimationGraph)
            .map(AssetImportOutcome::new)
            .map_err(AssetImportError::Parse);
    }
    if lower_name.ends_with(".state_machine.zranim") {
        return AnimationStateMachineAsset::from_bytes(&context.source_bytes)
            .map(ImportedAsset::AnimationStateMachine)
            .map(AssetImportOutcome::new)
            .map_err(AssetImportError::Parse);
    }

    Err(AssetImportError::UnsupportedFormat(format!(
        "unknown animation asset suffix for {}",
        context.source_path.display()
    )))
}

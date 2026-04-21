use std::fs;
use std::path::Path;

use super::AssetImporter;
use crate::asset::assets::ImportedAsset;
use crate::asset::{
    AnimationClipAsset, AnimationGraphAsset, AnimationSequenceAsset, AnimationSkeletonAsset,
    AnimationStateMachineAsset, AssetImportError,
};

impl AssetImporter {
    pub fn import_animation_asset(
        &self,
        source_path: &Path,
    ) -> Result<ImportedAsset, AssetImportError> {
        let lower_name = source_path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default()
            .to_ascii_lowercase();
        let bytes = fs::read(source_path)?;

        if lower_name.ends_with(".skeleton.zranim") {
            return AnimationSkeletonAsset::from_bytes(&bytes)
                .map(ImportedAsset::AnimationSkeleton)
                .map_err(AssetImportError::Parse);
        }
        if lower_name.ends_with(".clip.zranim") {
            return AnimationClipAsset::from_bytes(&bytes)
                .map(ImportedAsset::AnimationClip)
                .map_err(AssetImportError::Parse);
        }
        if lower_name.ends_with(".sequence.zranim") {
            return AnimationSequenceAsset::from_bytes(&bytes)
                .map(ImportedAsset::AnimationSequence)
                .map_err(AssetImportError::Parse);
        }
        if lower_name.ends_with(".graph.zranim") {
            return AnimationGraphAsset::from_bytes(&bytes)
                .map(ImportedAsset::AnimationGraph)
                .map_err(AssetImportError::Parse);
        }
        if lower_name.ends_with(".state_machine.zranim") {
            return AnimationStateMachineAsset::from_bytes(&bytes)
                .map(ImportedAsset::AnimationStateMachine)
                .map_err(AssetImportError::Parse);
        }

        Err(AssetImportError::UnsupportedFormat(format!(
            "unknown animation asset suffix for {}",
            source_path.display()
        )))
    }
}

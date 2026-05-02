use crate::asset::assets::{ImportedAsset, SoundAsset};
use crate::asset::{AssetImportContext, AssetImportError, AssetImportOutcome};

pub(crate) fn import_sound(
    context: &AssetImportContext,
) -> Result<AssetImportOutcome, AssetImportError> {
    let asset =
        SoundAsset::from_wav_bytes(&context.uri, &context.source_bytes).map_err(|error| {
            AssetImportError::Parse(format!(
                "decode wav {}: {error}",
                context.source_path.display()
            ))
        })?;
    Ok(AssetImportOutcome::new(ImportedAsset::Sound(asset)))
}

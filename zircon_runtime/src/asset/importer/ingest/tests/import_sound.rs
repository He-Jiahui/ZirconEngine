use crate::asset::assets::{ImportedAsset, SoundAsset};
use crate::asset::{AssetImportContext, AssetImportError, AssetImportOutcome};

// WAV 插件描述符的解码入口；以 source URI 构造根 Sound 资产，失败时附源路径供导入作业定位。
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
    Ok(AssetImportOutcome::new(
        context.uri.clone(),
        ImportedAsset::Sound(asset),
    ))
}

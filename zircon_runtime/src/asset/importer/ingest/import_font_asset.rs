use crate::asset::assets::{FontAsset, ImportedAsset};
use crate::asset::{AssetImportContext, AssetImportError, AssetImportOutcome};

pub(crate) fn import_font_asset(
    context: &AssetImportContext,
) -> Result<AssetImportOutcome, AssetImportError> {
    let document = context.source_text()?;
    let asset = FontAsset::from_toml_str(&document)
        .map_err(|error| AssetImportError::Parse(format!("parse font toml: {error}")))?;
    Ok(AssetImportOutcome::new(ImportedAsset::Font(asset)))
}

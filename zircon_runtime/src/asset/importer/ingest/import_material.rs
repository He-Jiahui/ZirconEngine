use crate::asset::assets::{ImportedAsset, MaterialAsset};
use crate::asset::{AssetImportContext, AssetImportError, AssetImportOutcome};

pub(crate) fn import_material(
    context: &AssetImportContext,
) -> Result<AssetImportOutcome, AssetImportError> {
    let document = context.source_text()?;
    let material = MaterialAsset::from_toml_str(&document)
        .map_err(|error| AssetImportError::Parse(format!("parse material toml: {error}")))?;
    Ok(AssetImportOutcome::new(ImportedAsset::Material(material)))
}

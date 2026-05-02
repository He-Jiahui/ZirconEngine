use crate::asset::{
    AssetImportContext, AssetImportError, AssetImportOutcome, DataAsset, DataAssetFormat,
    ImportedAsset,
};

pub(crate) fn import_plain_toml_data(
    context: &AssetImportContext,
) -> Result<AssetImportOutcome, AssetImportError> {
    let text = context.source_text()?;
    let value: toml::Value = toml::from_str(&text)
        .map_err(|error| AssetImportError::Parse(format!("parse toml data: {error}")))?;
    let canonical_json = serde_json::to_value(value)?;
    Ok(AssetImportOutcome::new(ImportedAsset::Data(DataAsset {
        uri: context.uri.clone(),
        format: DataAssetFormat::Toml,
        text,
        canonical_json,
    })))
}

pub(crate) fn import_json_data(
    context: &AssetImportContext,
) -> Result<AssetImportOutcome, AssetImportError> {
    let text = context.source_text()?;
    let canonical_json = serde_json::from_str(&text)
        .map_err(|error| AssetImportError::Parse(format!("parse json data: {error}")))?;
    Ok(AssetImportOutcome::new(ImportedAsset::Data(DataAsset {
        uri: context.uri.clone(),
        format: DataAssetFormat::Json,
        text,
        canonical_json,
    })))
}

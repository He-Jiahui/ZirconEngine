//! 通用数据资产同时保留原文与规范 JSON 表示，供编辑器展示和运行时读取使用；
//! 纯文本没有结构化值，以 Null 表示，不能将它误当作 JSON 文档。

use crate::asset::{
    AssetImportContext, AssetImportError, AssetImportOutcome, DataAsset, DataAssetFormat,
    ImportedAsset,
};

pub(crate) fn import_plain_toml_data(
    context: &AssetImportContext,
) -> Result<AssetImportOutcome, AssetImportError> {
    let source = context.source_str()?;
    let value: toml::Value =
        toml::from_str(source).map_err(|source| AssetImportError::TomlDeserialize {
            context: "parsing TOML data asset",
            source,
        })?;
    let canonical_json = serde_json::to_value(value)?;
    let text = source.to_owned();
    Ok(AssetImportOutcome::new(
        context.uri.clone(),
        ImportedAsset::Data(DataAsset {
            uri: context.uri.clone(),
            format: DataAssetFormat::Toml,
            text,
            canonical_json,
        }),
    ))
}

pub(crate) fn import_json_data(
    context: &AssetImportContext,
) -> Result<AssetImportOutcome, AssetImportError> {
    let source = context.source_str()?;
    let canonical_json =
        serde_json::from_str(source).map_err(|source| AssetImportError::JsonDeserialize {
            context: "parsing JSON data asset",
            source,
        })?;
    let text = source.to_owned();
    Ok(AssetImportOutcome::new(
        context.uri.clone(),
        ImportedAsset::Data(DataAsset {
            uri: context.uri.clone(),
            format: DataAssetFormat::Json,
            text,
            canonical_json,
        }),
    ))
}

pub(crate) fn import_text_data(
    context: &AssetImportContext,
) -> Result<AssetImportOutcome, AssetImportError> {
    let text = context.source_text()?;
    Ok(AssetImportOutcome::new(
        context.uri.clone(),
        ImportedAsset::Data(DataAsset {
            uri: context.uri.clone(),
            format: DataAssetFormat::Text,
            text,
            canonical_json: serde_json::Value::Null,
        }),
    ))
}

#[cfg(test)]
#[path = "tests/import_data_asset.rs"]
mod tests;

use crate::asset::{
    AssetImportContext, AssetImportError, AssetImportOutcome, DataAsset, DataAssetFormat,
    ImportedAsset,
};

pub(crate) fn import_plain_toml_data(
    context: &AssetImportContext,
) -> Result<AssetImportOutcome, AssetImportError> {
    let text = context.source_text()?;
    let value: toml::Value =
        toml::from_str(&text).map_err(|source| AssetImportError::TomlDeserialize {
            context: "parsing TOML data asset",
            source,
        })?;
    let canonical_json = serde_json::to_value(value)?;
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
    let text = context.source_text()?;
    let canonical_json =
        serde_json::from_str(&text).map_err(|source| AssetImportError::JsonDeserialize {
            context: "parsing JSON data asset",
            source,
        })?;
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
mod tests {
    use std::error::Error as _;

    use super::*;

    #[test]
    fn source_text_decode_error_retains_utf8_source() {
        let context = data_context("invalid.txt", vec![0xff]);

        let error = import_error(import_text_data(&context));

        match &error {
            AssetImportError::SourceTextDecode { path, .. } => {
                assert_eq!(path, &std::path::PathBuf::from("invalid.txt"));
            }
            other => panic!("expected source text decode error, got {other:?}"),
        }
        assert!(error.source().is_some());
    }

    #[test]
    fn toml_data_parse_error_retains_toml_source() {
        let context = data_context("invalid.toml", b"value = [".to_vec());

        let error = import_error(import_plain_toml_data(&context));

        match &error {
            AssetImportError::TomlDeserialize { context, .. } => {
                assert_eq!(*context, "parsing TOML data asset");
            }
            other => panic!("expected TOML deserialize error, got {other:?}"),
        }
        assert!(error.source().is_some());
    }

    #[test]
    fn json_data_parse_error_retains_json_source() {
        let context = data_context("invalid.json", b"{".to_vec());

        let error = import_error(import_json_data(&context));

        match &error {
            AssetImportError::JsonDeserialize { context, .. } => {
                assert_eq!(*context, "parsing JSON data asset");
            }
            other => panic!("expected JSON deserialize error, got {other:?}"),
        }
        assert!(error.source().is_some());
    }

    fn data_context(path: &str, source_bytes: Vec<u8>) -> AssetImportContext {
        AssetImportContext::new(
            path.into(),
            crate::asset::AssetUri::parse(&format!("res://data/{path}")).unwrap(),
            source_bytes,
            toml::Table::new(),
        )
    }

    fn import_error(result: Result<AssetImportOutcome, AssetImportError>) -> AssetImportError {
        match result {
            Ok(_) => panic!("invalid data import unexpectedly succeeded"),
            Err(error) => error,
        }
    }
}

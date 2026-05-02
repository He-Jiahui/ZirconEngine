use std::fmt;
use std::sync::Arc;

use serde::{Deserialize, Serialize};

use super::{
    AssetImportContext, AssetImportOutcome, AssetImporterDescriptor, AssetImporterHandler,
};
use crate::asset::{asset_kind_for_imported_asset, AssetImportError};
use crate::plugin::{
    LoadedNativePlugin, ZIRCON_NATIVE_PLUGIN_STATUS_DENIED, ZIRCON_NATIVE_PLUGIN_STATUS_ERROR,
    ZIRCON_NATIVE_PLUGIN_STATUS_OK, ZIRCON_NATIVE_PLUGIN_STATUS_PANIC,
};

const REQUEST_MAGIC: &[u8] = b"ZRIMP001\n";
const RESPONSE_MAGIC: &[u8] = b"ZRIMO001\n";

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NativeAssetImportRequestMetadata {
    pub importer_id: String,
    pub source_uri: String,
    pub source_path: String,
    #[serde(default)]
    pub import_settings: toml::Table,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NativeAssetImportResponseMetadata {
    pub importer_id: String,
    pub imported_asset: crate::asset::ImportedAsset,
    #[serde(default)]
    pub diagnostics: Vec<String>,
}

#[derive(Clone)]
pub struct NativeAssetImporterHandler {
    descriptor: AssetImporterDescriptor,
    plugin: Arc<LoadedNativePlugin>,
}

impl NativeAssetImporterHandler {
    pub fn new(descriptor: AssetImporterDescriptor, plugin: Arc<LoadedNativePlugin>) -> Self {
        Self { descriptor, plugin }
    }
}

impl fmt::Debug for NativeAssetImporterHandler {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("NativeAssetImporterHandler")
            .field("descriptor", &self.descriptor)
            .field("plugin_id", &self.plugin.plugin_id)
            .finish_non_exhaustive()
    }
}

impl AssetImporterHandler for NativeAssetImporterHandler {
    fn descriptor(&self) -> &AssetImporterDescriptor {
        &self.descriptor
    }

    fn import(&self, context: &AssetImportContext) -> Result<AssetImportOutcome, AssetImportError> {
        let command = format!("asset.import/{}", self.descriptor.id);
        let request = encode_request(
            &NativeAssetImportRequestMetadata {
                importer_id: self.descriptor.id.clone(),
                source_uri: context.uri.to_string(),
                source_path: context.source_path.to_string_lossy().into_owned(),
                import_settings: context.import_settings.clone(),
            },
            &context.source_bytes,
        )?;
        let report = self.plugin.invoke_runtime_command(&command, &request);
        let status = report.status_code;
        let payload = report.payload.ok_or_else(|| {
            native_status_error(status, "native importer did not return an output payload")
        })?;
        if status != ZIRCON_NATIVE_PLUGIN_STATUS_OK {
            return Err(native_status_error(status, &report.diagnostics.join("; ")));
        }
        let response = decode_response(&payload)?;
        if response.importer_id != self.descriptor.id {
            return Err(AssetImportError::Native(format!(
                "native importer response id {} did not match {}",
                response.importer_id, self.descriptor.id
            )));
        }
        let actual_kind = asset_kind_for_imported_asset(&response.imported_asset);
        if !self.descriptor.allows_output_kind(actual_kind) {
            return Err(AssetImportError::Native(format!(
                "native importer {} returned {actual_kind:?}, expected {:?}",
                self.descriptor.id, self.descriptor.output_kind
            )));
        }
        let mut outcome = AssetImportOutcome::new(response.imported_asset);
        outcome.diagnostics.extend(
            response
                .diagnostics
                .into_iter()
                .map(|message| crate::core::resource::ResourceDiagnostic::error(message)),
        );
        Ok(outcome)
    }
}

pub fn encode_request(
    metadata: &NativeAssetImportRequestMetadata,
    source_bytes: &[u8],
) -> Result<Vec<u8>, AssetImportError> {
    encode_envelope(REQUEST_MAGIC, metadata, source_bytes)
}

pub fn decode_response(
    payload: &[u8],
) -> Result<NativeAssetImportResponseMetadata, AssetImportError> {
    let (metadata, artifact_bytes) =
        decode_envelope::<NativeAssetImportResponseMetadata>(RESPONSE_MAGIC, payload)?;
    if !artifact_bytes.is_empty() {
        return Err(AssetImportError::Native(
            "native importer response artifact bytes are reserved for future payloads".to_string(),
        ));
    }
    Ok(metadata)
}

fn encode_envelope<T: Serialize>(
    magic: &[u8],
    metadata: &T,
    bytes: &[u8],
) -> Result<Vec<u8>, AssetImportError> {
    let metadata = serde_json::to_vec(metadata)?;
    let mut envelope = Vec::with_capacity(magic.len() + 8 + metadata.len() + bytes.len());
    envelope.extend_from_slice(magic);
    envelope.extend_from_slice(&(metadata.len() as u64).to_le_bytes());
    envelope.extend_from_slice(&metadata);
    envelope.extend_from_slice(bytes);
    Ok(envelope)
}

fn decode_envelope<'payload, T: for<'de> Deserialize<'de>>(
    magic: &[u8],
    payload: &'payload [u8],
) -> Result<(T, &'payload [u8]), AssetImportError> {
    if !payload.starts_with(magic) || payload.len() < magic.len() + 8 {
        return Err(AssetImportError::Native(
            "native importer envelope magic is missing or malformed".to_string(),
        ));
    }
    let len_start = magic.len();
    let len_end = len_start + 8;
    let metadata_len = u64::from_le_bytes(payload[len_start..len_end].try_into().unwrap()) as usize;
    let metadata_end = len_end + metadata_len;
    if metadata_end > payload.len() {
        return Err(AssetImportError::Native(
            "native importer envelope metadata length exceeds payload".to_string(),
        ));
    }
    let metadata = serde_json::from_slice(&payload[len_end..metadata_end])?;
    Ok((metadata, &payload[metadata_end..]))
}

fn native_status_error(status: u32, detail: &str) -> AssetImportError {
    let status_name = match status {
        ZIRCON_NATIVE_PLUGIN_STATUS_OK => "ok",
        ZIRCON_NATIVE_PLUGIN_STATUS_ERROR => "error",
        ZIRCON_NATIVE_PLUGIN_STATUS_DENIED => "denied",
        ZIRCON_NATIVE_PLUGIN_STATUS_PANIC => "panic",
        _ => "unknown",
    };
    AssetImportError::Native(format!(
        "native importer command returned {status_name}: {detail}"
    ))
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::asset::{AssetUri, DataAsset, DataAssetFormat, ImportedAsset};

    #[test]
    fn native_import_request_envelope_roundtrips_metadata_and_source_bytes() {
        let metadata = NativeAssetImportRequestMetadata {
            importer_id: "fixture.data".to_string(),
            source_uri: "res://assets/weather.fixture".to_string(),
            source_path: "assets/weather.fixture".to_string(),
            import_settings: toml::Table::new(),
        };

        let encoded = encode_request(&metadata, b"source bytes").expect("encoded request");
        let (decoded, source_bytes) =
            decode_envelope::<NativeAssetImportRequestMetadata>(REQUEST_MAGIC, &encoded)
                .expect("decoded request");

        assert_eq!(decoded, metadata);
        assert_eq!(source_bytes, b"source bytes");
    }

    #[test]
    fn native_import_response_envelope_rejects_malformed_magic() {
        let error = decode_response(b"wrong magic").expect_err("malformed envelope");

        assert!(error.to_string().contains("envelope magic"));
    }

    #[test]
    fn native_import_response_envelope_decodes_neutral_asset_dto() {
        let metadata = NativeAssetImportResponseMetadata {
            importer_id: "fixture.data".to_string(),
            imported_asset: ImportedAsset::Data(DataAsset {
                uri: AssetUri::parse("res://assets/weather.fixture").unwrap(),
                format: DataAssetFormat::Json,
                text: "{\"temperature\":21}".to_string(),
                canonical_json: json!({ "temperature": 21 }),
            }),
            diagnostics: vec!["fixture diagnostic".to_string()],
        };
        let encoded = encode_envelope(RESPONSE_MAGIC, &metadata, &[]).expect("encoded response");

        let decoded = decode_response(&encoded).expect("decoded response");

        assert_eq!(decoded, metadata);
    }
}

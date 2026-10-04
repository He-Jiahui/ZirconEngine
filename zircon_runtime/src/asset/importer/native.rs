use std::fmt;
use std::sync::Arc;

use serde::{Deserialize, Serialize};

use super::{
    AssetImportContext, AssetImportOutcome, AssetImporterDescriptor, AssetImporterHandler,
    AssetSchemaMigrationReport, ImportedAssetEntry,
};
use crate::asset::{asset_kind_for_imported_asset, AssetImportError, AssetUri};

const REQUEST_MAGIC: &[u8] = b"ZRIMP001\n";
const RESPONSE_MAGIC: &[u8] = b"ZRIMO002\n";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NativeAssetImportCommandStatus {
    Ok,
    Error,
    Denied,
    Panic,
    Unknown(u32),
}

impl NativeAssetImportCommandStatus {
    fn label(self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::Error => "error",
            Self::Denied => "denied",
            Self::Panic => "panic",
            Self::Unknown(_) => "unknown",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativeAssetImportCommandReport {
    pub status: NativeAssetImportCommandStatus,
    pub diagnostics: Vec<String>,
    pub payload: Option<Vec<u8>>,
}

pub trait NativeAssetImportCommandHost: Send + Sync {
    fn command_host_id(&self) -> &str;

    fn invoke_asset_import_command(
        &self,
        command: &str,
        payload: &[u8],
    ) -> NativeAssetImportCommandReport;
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NativeAssetImportRequestMetadata {
    pub importer_id: String,
    pub source_uri: String,
    pub source_path: String,
    #[serde(default)]
    pub import_settings: toml::Table,
}

#[derive(Serialize)]
struct BorrowedNativeAssetImportRequestMetadata<'a> {
    importer_id: &'a str,
    source_uri: &'a AssetUri,
    source_path: &'a str,
    import_settings: &'a toml::Table,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NativeAssetImportResponseMetadata {
    pub importer_id: String,
    pub entries: Vec<NativeAssetImportEntryMetadata>,
    /// Resolution observations produced while decoding this source.
    ///
    /// The host only publishes these observations. It does not grant a native importer
    /// permission to rewrite a stable asset identity or project source document.
    pub reference_repairs: Vec<crate::asset::ReferenceRepair>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NativeAssetImportEntryMetadata {
    pub locator: AssetUri,
    pub imported_asset: crate::asset::ImportedAsset,
    #[serde(default)]
    pub dependencies: Vec<crate::asset::AssetUri>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub migration_report: Option<AssetSchemaMigrationReport>,
    #[serde(default)]
    pub diagnostics: Vec<String>,
}

#[derive(Clone)]
pub struct NativeAssetImporterHandler {
    descriptor: AssetImporterDescriptor,
    command: Box<str>,
    command_host: Arc<dyn NativeAssetImportCommandHost>,
}

impl NativeAssetImporterHandler {
    pub fn new(
        descriptor: AssetImporterDescriptor,
        command_host: Arc<dyn NativeAssetImportCommandHost>,
    ) -> Self {
        let command = format!("asset.import/{}", descriptor.id).into_boxed_str();
        Self {
            descriptor,
            command,
            command_host,
        }
    }
}

impl fmt::Debug for NativeAssetImporterHandler {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("NativeAssetImporterHandler")
            .field("descriptor", &self.descriptor)
            .field("command_host_id", &self.command_host.command_host_id())
            .finish_non_exhaustive()
    }
}

impl AssetImporterHandler for NativeAssetImporterHandler {
    fn descriptor(&self) -> &AssetImporterDescriptor {
        &self.descriptor
    }

    fn import(&self, context: &AssetImportContext) -> Result<AssetImportOutcome, AssetImportError> {
        let request = encode_borrowed_request(&self.descriptor, context)?;
        let report = self
            .command_host
            .invoke_asset_import_command(&self.command, &request);
        let payload = native_command_payload(report)?;
        let response = decode_response(&payload)?;
        native_response_to_outcome(&self.descriptor, response)
    }
}

pub fn encode_request(
    metadata: &NativeAssetImportRequestMetadata,
    source_bytes: &[u8],
) -> Result<Vec<u8>, AssetImportError> {
    encode_envelope(REQUEST_MAGIC, metadata, source_bytes)
}

fn encode_borrowed_request(
    descriptor: &AssetImporterDescriptor,
    context: &AssetImportContext,
) -> Result<Vec<u8>, AssetImportError> {
    let source_path = context.source_path.to_string_lossy();
    encode_envelope(
        REQUEST_MAGIC,
        &BorrowedNativeAssetImportRequestMetadata {
            importer_id: &descriptor.id,
            source_uri: &context.uri,
            source_path: source_path.as_ref(),
            import_settings: context.import_settings(),
        },
        &context.source_bytes,
    )
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

fn native_status_error(status: NativeAssetImportCommandStatus, detail: &str) -> AssetImportError {
    AssetImportError::Native(format!(
        "native importer command returned {}: {detail}",
        status.label()
    ))
}

fn native_command_payload(
    report: NativeAssetImportCommandReport,
) -> Result<Vec<u8>, AssetImportError> {
    let status = report.status;
    if status != NativeAssetImportCommandStatus::Ok {
        let detail = if report.diagnostics.is_empty() {
            "native importer returned no diagnostics".to_string()
        } else {
            report.diagnostics.join("; ")
        };
        return Err(native_status_error(status, &detail));
    }
    report.payload.ok_or_else(|| {
        native_status_error(status, "native importer did not return an output payload")
    })
}

fn native_response_to_outcome(
    descriptor: &AssetImporterDescriptor,
    response: NativeAssetImportResponseMetadata,
) -> Result<AssetImportOutcome, AssetImportError> {
    if response.importer_id != descriptor.id {
        return Err(AssetImportError::Native(format!(
            "native importer response id {} did not match {}",
            response.importer_id, descriptor.id
        )));
    }
    if response.entries.is_empty() {
        return Err(AssetImportError::Native(format!(
            "native importer {} returned no imported asset entries",
            descriptor.id
        )));
    }
    for entry in &response.entries {
        let actual_kind = asset_kind_for_imported_asset(&entry.imported_asset);
        if !descriptor.allows_output_kind(actual_kind) {
            return Err(AssetImportError::Native(format!(
                "native importer {} returned {actual_kind:?}, expected {:?}",
                descriptor.id, descriptor.output_kind
            )));
        }
    }
    Ok(AssetImportOutcome {
        entries: response
            .entries
            .into_iter()
            .map(|entry| {
                let mut imported = ImportedAssetEntry::new(entry.locator, entry.imported_asset);
                imported.dependencies = entry.dependencies;
                imported.migration_report = entry.migration_report;
                imported.diagnostics.extend(
                    entry
                        .diagnostics
                        .into_iter()
                        .map(|message| crate::core::resource::ResourceDiagnostic::error(message)),
                );
                imported
            })
            .collect(),
        reference_repairs: response.reference_repairs,
    })
}

#[cfg(test)]
#[path = "tests/native.rs"]
mod tests;

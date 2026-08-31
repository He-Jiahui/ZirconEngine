use std::fmt;
use std::sync::Arc;

use serde::{Deserialize, Serialize};

use super::{
    AssetImportContext, AssetImportOutcome, AssetImporterDescriptor, AssetImporterHandler,
    AssetSchemaMigrationReport, ImportedAssetEntry,
};
use crate::asset::{AssetImportError, AssetUri, asset_kind_for_imported_asset};

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
            import_settings: &context.import_settings,
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
mod tests {
    use std::hint::black_box;
    use std::path::PathBuf;
    use std::time::Instant;

    use serde_json::json;

    use super::*;
    use crate::asset::{AssetKind, AssetUri, DataAsset, DataAssetFormat, ImportedAsset};

    const SAMPLE_PAIRS: usize = 21;
    const REQUESTS_PER_SAMPLE: usize = 256;

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
            entries: vec![NativeAssetImportEntryMetadata {
                locator: AssetUri::parse("res://assets/weather.fixture").unwrap(),
                imported_asset: ImportedAsset::Data(DataAsset {
                    uri: AssetUri::parse("res://assets/weather.fixture").unwrap(),
                    format: DataAssetFormat::Json,
                    text: "{\"temperature\":21}".to_string(),
                    canonical_json: json!({ "temperature": 21 }),
                }),
                dependencies: vec![AssetUri::parse("res://assets/dependency.fixture").unwrap()],
                migration_report: Some(AssetSchemaMigrationReport {
                    source_schema_version: Some(1),
                    target_schema_version: 2,
                    summary: "fixture migrated to schema 2".to_string(),
                }),
                diagnostics: vec!["fixture diagnostic".to_string()],
            }],
            reference_repairs: Vec::new(),
        };
        let encoded = encode_envelope(RESPONSE_MAGIC, &metadata, &[]).expect("encoded response");

        let decoded = decode_response(&encoded).expect("decoded response");

        assert_eq!(decoded, metadata);
    }

    #[test]
    fn native_import_response_envelope_rejects_reserved_artifact_bytes() {
        let metadata = fixture_native_response(
            "fixture.data",
            fixture_data().uri.clone(),
            ImportedAsset::Data(fixture_data()),
        );
        let encoded =
            encode_envelope(RESPONSE_MAGIC, &metadata, b"artifact").expect("encoded response");

        let error = decode_response(&encoded).expect_err("artifact bytes are reserved");

        assert!(error.to_string().contains("reserved"));
    }

    #[test]
    fn native_import_response_rejects_mismatched_importer_id() {
        let descriptor =
            AssetImporterDescriptor::new("fixture.data", "fixture", AssetKind::Data, 1)
                .with_source_extensions(["fixture"]);
        let response = fixture_native_response(
            "other.data",
            fixture_data().uri.clone(),
            ImportedAsset::Data(fixture_data()),
        );

        let error = native_response_to_outcome(&descriptor, response).expect_err("id mismatch");

        assert!(error.to_string().contains("did not match fixture.data"));
    }

    #[test]
    fn native_import_response_rejects_wrong_output_kind() {
        let descriptor =
            AssetImporterDescriptor::new("fixture.model", "fixture", AssetKind::Model, 1)
                .with_source_extensions(["fixture"]);
        let response = fixture_native_response(
            "fixture.model",
            AssetUri::parse("res://assets/weather.fixture").unwrap(),
            ImportedAsset::Data(fixture_data()),
        );

        let error = native_response_to_outcome(&descriptor, response).expect_err("wrong kind");

        assert!(error.to_string().contains("returned Data"));
        assert!(error.to_string().contains("expected Model"));
    }

    #[test]
    fn native_import_command_errors_preserve_status_diagnostics_without_payload() {
        let report = NativeAssetImportCommandReport {
            status: NativeAssetImportCommandStatus::Denied,
            diagnostics: vec!["denied native command unknown".to_string()],
            payload: None,
        };

        let error = native_command_payload(report).expect_err("denied status");

        assert!(error.to_string().contains("command returned denied"));
        assert!(error.to_string().contains("denied native command unknown"));
    }

    #[test]
    fn native_import_command_requires_payload_only_after_ok_status() {
        let report = NativeAssetImportCommandReport {
            status: NativeAssetImportCommandStatus::Ok,
            diagnostics: Vec::new(),
            payload: None,
        };

        let error = native_command_payload(report).expect_err("missing ok payload");

        assert!(
            error
                .to_string()
                .contains("did not return an output payload")
        );
    }

    #[test]
    fn native_import_response_converts_diagnostics_to_resource_diagnostics() {
        let descriptor =
            AssetImporterDescriptor::new("fixture.data", "fixture", AssetKind::Data, 1)
                .with_source_extensions(["fixture"]);
        let mut response = fixture_native_response(
            "fixture.data",
            fixture_data().uri.clone(),
            ImportedAsset::Data(fixture_data()),
        );
        response.entries[0]
            .diagnostics
            .push("native warning".to_string());

        let outcome = native_response_to_outcome(&descriptor, response).expect("valid response");

        assert!(
            outcome.entries[0]
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.message == "native warning")
        );
    }

    #[test]
    fn native_import_response_preserves_declared_dependencies() {
        let descriptor =
            AssetImporterDescriptor::new("fixture.data", "fixture", AssetKind::Data, 1)
                .with_source_extensions(["fixture"]);
        let dependency = AssetUri::parse("res://assets/dependency.fixture").unwrap();
        let mut response = fixture_native_response(
            "fixture.data",
            fixture_data().uri.clone(),
            ImportedAsset::Data(fixture_data()),
        );
        response.entries[0].dependencies.push(dependency.clone());

        let outcome = native_response_to_outcome(&descriptor, response).expect("valid response");

        assert_eq!(outcome.entries[0].dependencies, vec![dependency]);
    }

    #[test]
    fn native_import_response_preserves_schema_migration_report() {
        let descriptor =
            AssetImporterDescriptor::new("fixture.data", "fixture", AssetKind::Data, 1)
                .with_source_extensions(["fixture"]);
        let migration_report = AssetSchemaMigrationReport {
            source_schema_version: Some(1),
            target_schema_version: 3,
            summary: "native fixture migrated source schema".to_string(),
        };
        let mut response = fixture_native_response(
            "fixture.data",
            fixture_data().uri.clone(),
            ImportedAsset::Data(fixture_data()),
        );
        response.entries[0].migration_report = Some(migration_report.clone());

        let outcome = native_response_to_outcome(&descriptor, response).expect("valid response");

        assert_eq!(outcome.entries[0].migration_report, Some(migration_report));
    }

    #[test]
    fn native_import_response_preserves_reference_repair_observations() {
        let descriptor =
            AssetImporterDescriptor::new("fixture.data", "fixture", AssetKind::Data, 1);
        let repair = native_path_hint_repair();
        let mut response = fixture_native_response(
            "fixture.data",
            fixture_data().uri.clone(),
            ImportedAsset::Data(fixture_data()),
        );
        response.reference_repairs = vec![repair.clone()];

        let outcome = native_response_to_outcome(&descriptor, response).expect("valid response");

        assert_eq!(outcome.reference_repairs, vec![repair]);
    }

    #[test]
    fn importer_request_publish_contract_native_borrowed_wire() {
        let descriptor = benchmark_descriptor();
        let context = benchmark_context();
        let owned = NativeAssetImportRequestMetadata {
            importer_id: descriptor.id.clone(),
            source_uri: context.uri.to_string(),
            source_path: context.source_path.to_string_lossy().into_owned(),
            import_settings: context.import_settings.clone(),
        };

        let owned_bytes = encode_request(&owned, &context.source_bytes).unwrap();
        let borrowed_bytes = encode_borrowed_request(&descriptor, &context).unwrap();

        assert_eq!(borrowed_bytes, owned_bytes);
        let handler = NativeAssetImporterHandler::new(descriptor, Arc::new(PanicNativeCommandHost));
        assert_eq!(
            handler.command.as_ref(),
            "asset.import/plugins07.native.hotpath"
        );
    }

    #[test]
    #[ignore = "release performance gate"]
    fn importer_request_publish_performance_release_native_borrowed_metadata() {
        let descriptor = benchmark_descriptor();
        let context = benchmark_context();
        for _ in 0..4 {
            black_box(measure_owned_requests(&descriptor, &context));
            black_box(measure_borrowed_requests(&descriptor, &context));
        }
        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
        for pair_index in 0..SAMPLE_PAIRS {
            let (legacy_ns, optimized_ns) = if pair_index % 2 == 0 {
                (
                    measure_owned_requests(&descriptor, &context),
                    measure_borrowed_requests(&descriptor, &context),
                )
            } else {
                let optimized_ns = measure_borrowed_requests(&descriptor, &context);
                (measure_owned_requests(&descriptor, &context), optimized_ns)
            };
            legacy_samples.push(legacy_ns);
            optimized_samples.push(optimized_ns);
        }

        let legacy_p95 = nearest_rank_p95(&legacy_samples);
        let optimized_p95 = nearest_rank_p95(&optimized_samples);
        let improvement_percent =
            legacy_p95.saturating_sub(optimized_p95).saturating_mul(100) / legacy_p95.max(1);
        println!(
            "PERF_RESULT plugins07_native_borrowed_request sample_pairs={SAMPLE_PAIRS} requests_per_sample={REQUESTS_PER_SAMPLE} legacy_ns={} optimized_ns={} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} improvement_percent={improvement_percent} threshold_percent=20 legacy_request_field_clones_per_sample={} optimized_request_field_clones_per_sample=0 legacy_command_allocations_per_sample={REQUESTS_PER_SAMPLE} optimized_command_allocations_per_sample=0 order=alternating_legacy_first_even legacy_first_pairs=11 optimized_first_pairs=10",
            csv(&legacy_samples),
            csv(&optimized_samples),
            REQUESTS_PER_SAMPLE * 4,
        );
        assert!(
            improvement_percent >= 20,
            "borrowed native request preparation must improve P95 by at least 20%"
        );
    }

    #[derive(Debug)]
    struct PanicNativeCommandHost;

    impl NativeAssetImportCommandHost for PanicNativeCommandHost {
        fn command_host_id(&self) -> &str {
            "plugins07.panic"
        }

        fn invoke_asset_import_command(
            &self,
            _command: &str,
            _payload: &[u8],
        ) -> NativeAssetImportCommandReport {
            panic!("command host is not invoked by the request hotpath contract")
        }
    }

    fn benchmark_descriptor() -> AssetImporterDescriptor {
        AssetImporterDescriptor::new(
            "plugins07.native.hotpath",
            "plugins07.native",
            AssetKind::Data,
            1,
        )
        .with_source_extensions(["fixture"])
    }

    fn benchmark_context() -> AssetImportContext {
        let mut import_settings = toml::Table::new();
        for index in 0..64 {
            import_settings.insert(
                format!("setting_{index}"),
                toml::Value::String(format!("plugins07-value-{index:04}")),
            );
        }
        AssetImportContext::new(
            PathBuf::from("assets/native/plugins07/fixture.fixture"),
            AssetUri::parse("res://native/plugins07/fixture.fixture").unwrap(),
            b"plugins07 native source".to_vec(),
            import_settings,
        )
    }

    fn measure_owned_requests(
        descriptor: &AssetImporterDescriptor,
        context: &AssetImportContext,
    ) -> u128 {
        let started = Instant::now();
        for _ in 0..REQUESTS_PER_SAMPLE {
            let command = format!("asset.import/{}", black_box(&descriptor.id));
            let metadata = NativeAssetImportRequestMetadata {
                importer_id: black_box(&descriptor.id).clone(),
                source_uri: black_box(&context.uri).to_string(),
                source_path: black_box(&context.source_path)
                    .to_string_lossy()
                    .into_owned(),
                import_settings: black_box(&context.import_settings).clone(),
            };
            let request = encode_request(&metadata, black_box(&context.source_bytes)).unwrap();
            black_box((command, request));
        }
        started.elapsed().as_nanos().max(1)
    }

    fn measure_borrowed_requests(
        descriptor: &AssetImporterDescriptor,
        context: &AssetImportContext,
    ) -> u128 {
        let command = format!("asset.import/{}", descriptor.id).into_boxed_str();
        let started = Instant::now();
        for _ in 0..REQUESTS_PER_SAMPLE {
            let request =
                encode_borrowed_request(black_box(descriptor), black_box(context)).unwrap();
            black_box((&command, request));
        }
        started.elapsed().as_nanos().max(1)
    }

    fn nearest_rank_p95(samples: &[u128]) -> u128 {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        let rank = (sorted.len() * 95).div_ceil(100);
        sorted[rank.saturating_sub(1)]
    }

    fn csv(samples: &[u128]) -> String {
        samples
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }

    fn fixture_native_response(
        importer_id: &str,
        locator: AssetUri,
        imported_asset: ImportedAsset,
    ) -> NativeAssetImportResponseMetadata {
        NativeAssetImportResponseMetadata {
            importer_id: importer_id.to_string(),
            entries: vec![NativeAssetImportEntryMetadata {
                locator,
                imported_asset,
                dependencies: Vec::new(),
                migration_report: None,
                diagnostics: Vec::new(),
            }],
            reference_repairs: Vec::new(),
        }
    }

    fn native_path_hint_repair() -> crate::asset::ReferenceRepair {
        use zircon_runtime_interface::project::{AssetRef, RelPath};

        let guid = "9a111111-2222-4333-8444-555555555555".parse().unwrap();
        crate::asset::ReferenceRepair {
            stale: AssetRef::try_new(
                guid,
                RelPath::parse("assets/data/legacy.fixture").unwrap(),
                None,
            )
            .unwrap(),
            resolved: AssetRef::try_new(
                guid,
                RelPath::parse("assets/data/current.fixture").unwrap(),
                None,
            )
            .unwrap(),
            kind: crate::asset::ReferenceRepairKind::PathHint,
        }
    }

    fn fixture_data() -> DataAsset {
        DataAsset {
            uri: AssetUri::parse("res://assets/weather.fixture").unwrap(),
            format: DataAssetFormat::Json,
            text: "{\"temperature\":21}".to_string(),
            canonical_json: json!({ "temperature": 21 }),
        }
    }
}

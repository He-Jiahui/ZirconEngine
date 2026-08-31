use std::ffi::CStr;
use std::hint::black_box;
use std::time::Instant;

use serde_json::json;

use super::{
    decode_import_request, encode_import_response, NativeAssetImportRequestMetadata,
    IMPORT_ENVELOPE_LENGTH_BYTES, IMPORT_REQUEST_MAGIC, IMPORT_RESPONSE_MAGIC,
    MAX_IMPORT_SOURCE_BYTES, NATIVE_DYNAMIC_FIXTURE_DECLARATION, NATIVE_EDITOR_ENTRY,
    NATIVE_EDITOR_REGISTRATION_MANIFEST, NATIVE_PLUGIN_ID, NATIVE_REQUESTED_CAPABILITIES,
    NATIVE_RUNTIME_ENTRY, NATIVE_RUNTIME_REGISTRATION_MANIFEST, PLUGIN_MANIFEST,
};

const BENCHMARK_SAMPLE_PAIRS: usize = 21;
const BENCHMARK_ITERATIONS: usize = 8;
const BENCHMARK_TIME_RATIO_THRESHOLD_BPS: u128 = 11_000;

fn baseline_encode_import_response(
    metadata: &NativeAssetImportRequestMetadata,
    source_bytes: &[u8],
) -> Result<Vec<u8>, String> {
    let text = std::str::from_utf8(source_bytes).map_err(|error| error.to_string())?;
    let canonical_json: serde_json::Value =
        serde_json::from_str(text).map_err(|error| error.to_string())?;
    let response_metadata = json!({
        "importer_id": metadata.importer_id,
        "entries": [
            {
                "locator": metadata.source_uri,
                "imported_asset": {
                    "Data": {
                        "uri": metadata.source_uri,
                        "format": "json",
                        "text": text,
                        "canonical_json": canonical_json,
                    }
                },
                "migration_report": {
                    "source_schema_version": 1,
                    "target_schema_version": 2,
                    "summary": format!("native fixture migrated {}", metadata.source_path),
                },
                "diagnostics": [
                    format!("native fixture imported {}", metadata.source_path),
                ],
            }
        ],
        "reference_repairs": [],
    });
    let metadata_bytes =
        serde_json::to_vec(&response_metadata).map_err(|error| error.to_string())?;
    let mut response = Vec::with_capacity(
        IMPORT_RESPONSE_MAGIC.len() + IMPORT_ENVELOPE_LENGTH_BYTES + metadata_bytes.len(),
    );
    response.extend_from_slice(IMPORT_RESPONSE_MAGIC);
    response.extend_from_slice(&(metadata_bytes.len() as u64).to_le_bytes());
    response.extend_from_slice(&metadata_bytes);
    Ok(response)
}

fn response_metadata(response: &[u8]) -> serde_json::Value {
    assert!(response.starts_with(IMPORT_RESPONSE_MAGIC));
    let length_start = IMPORT_RESPONSE_MAGIC.len();
    let metadata_start = length_start + IMPORT_ENVELOPE_LENGTH_BYTES;
    let metadata_len = usize::try_from(u64::from_le_bytes(
        response[length_start..metadata_start].try_into().unwrap(),
    ))
    .unwrap();
    serde_json::from_slice(&response[metadata_start..metadata_start + metadata_len]).unwrap()
}

fn measure_response_encoder(iterations: usize, mut encode: impl FnMut() -> Vec<u8>) -> u128 {
    let timer = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..iterations {
        checksum ^= black_box(encode()).len();
    }
    black_box(checksum);
    timer.elapsed().as_nanos()
}

fn nearest_rank_p95(samples: &mut [u128]) -> u128 {
    samples.sort_unstable();
    samples[(samples.len() * 95 - 1) / 100]
}

fn sample_csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}

#[test]
fn import_request_rejects_overflowing_metadata_length_without_panicking() {
    let mut payload = IMPORT_REQUEST_MAGIC.to_vec();
    payload.extend_from_slice(&u64::MAX.to_le_bytes());

    let decoded = std::panic::catch_unwind(|| decode_import_request(&payload));

    assert!(decoded.is_ok(), "malformed wire lengths must not panic");
    assert!(decoded.unwrap().is_err());
}

#[test]
fn import_response_encoder_does_not_materialize_an_owned_response_tree() {
    let source = include_str!("lib.rs");
    let encoder = source
        .split_once("fn encode_import_response(")
        .expect("response encoder")
        .1
        .split_once("unsafe extern \"C\" fn fixture_save_state")
        .expect("response encoder boundary")
        .0;

    assert!(!encoder.contains("json!("));
    assert!(encoder.contains("serde_json::to_writer"));
}

#[test]
fn bounded_import_response_matches_the_current_protocol_baseline() {
    let metadata = NativeAssetImportRequestMetadata {
        importer_id: "native_dynamic_fixture.data_json".to_string(),
        source_uri: "res://assets/weather.nativejson".to_string(),
        source_path: "weather.nativejson".to_string(),
    };
    let source = br#"{"temperature":21,"condition":"clear"}"#;

    let baseline = baseline_encode_import_response(&metadata, source).unwrap();
    let bounded = encode_import_response(&metadata, source, 1024 * 1024).unwrap();

    assert_eq!(response_metadata(&bounded), response_metadata(&baseline));
}

#[test]
fn import_response_stops_before_exceeding_the_host_output_budget() {
    let metadata = NativeAssetImportRequestMetadata {
        importer_id: "native_dynamic_fixture.data_json".to_string(),
        source_uri: "res://assets/weather.nativejson".to_string(),
        source_path: "weather.nativejson".to_string(),
    };

    let error = encode_import_response(&metadata, br#"{"temperature":21}"#, 32)
        .expect_err("the bounded writer must reject an oversized response");

    assert!(error.contains("host output budget"));
}

#[test]
fn import_request_rejects_source_beyond_the_fixture_budget() {
    let metadata = br#"{"importer_id":"native_dynamic_fixture.data_json","source_uri":"res://large","source_path":"large.nativejson"}"#;
    let mut payload = IMPORT_REQUEST_MAGIC.to_vec();
    payload.extend_from_slice(&(metadata.len() as u64).to_le_bytes());
    payload.extend_from_slice(metadata);
    payload.resize(payload.len() + MAX_IMPORT_SOURCE_BYTES + 1, b' ');

    let error = decode_import_request(&payload).unwrap_err();

    assert!(error.contains("source exceeds the fixture budget"));
}

#[test]
#[ignore = "release-only performance evidence"]
fn benchmark_bounded_native_import_response() {
    let metadata = NativeAssetImportRequestMetadata {
        importer_id: "native_dynamic_fixture.data_json".to_string(),
        source_uri: "res://assets/large.nativejson".to_string(),
        source_path: "large.nativejson".to_string(),
    };
    let source = format!(
        r#"{{"payload":"{}","enabled":true}}"#,
        "x".repeat(128 * 1024)
    );
    let baseline_once = baseline_encode_import_response(&metadata, source.as_bytes()).unwrap();
    let bounded_once = encode_import_response(&metadata, source.as_bytes(), 1024 * 1024).unwrap();
    assert_eq!(
        response_metadata(&bounded_once),
        response_metadata(&baseline_once)
    );

    let mut baseline_samples = Vec::with_capacity(BENCHMARK_SAMPLE_PAIRS);
    let mut bounded_samples = Vec::with_capacity(BENCHMARK_SAMPLE_PAIRS);
    for sample_index in 0..BENCHMARK_SAMPLE_PAIRS {
        if sample_index % 2 == 0 {
            baseline_samples.push(measure_response_encoder(BENCHMARK_ITERATIONS, || {
                baseline_encode_import_response(&metadata, black_box(source.as_bytes())).unwrap()
            }));
            bounded_samples.push(measure_response_encoder(BENCHMARK_ITERATIONS, || {
                encode_import_response(&metadata, black_box(source.as_bytes()), 1024 * 1024)
                    .unwrap()
            }));
        } else {
            bounded_samples.push(measure_response_encoder(BENCHMARK_ITERATIONS, || {
                encode_import_response(&metadata, black_box(source.as_bytes()), 1024 * 1024)
                    .unwrap()
            }));
            baseline_samples.push(measure_response_encoder(BENCHMARK_ITERATIONS, || {
                baseline_encode_import_response(&metadata, black_box(source.as_bytes())).unwrap()
            }));
        }
    }

    let baseline_raw = baseline_samples.clone();
    let bounded_raw = bounded_samples.clone();
    let baseline_p95_ns = nearest_rank_p95(&mut baseline_samples);
    let bounded_p95_ns = nearest_rank_p95(&mut bounded_samples);
    let ratio_bps = bounded_p95_ns.saturating_mul(10_000) / baseline_p95_ns.max(1);
    let baseline_intermediate_metadata_bytes =
        (baseline_once.len() - IMPORT_RESPONSE_MAGIC.len() - IMPORT_ENVELOPE_LENGTH_BYTES)
            * BENCHMARK_ITERATIONS;
    let baseline_source_text_clone_bytes = source.len() * BENCHMARK_ITERATIONS;

    println!(
        "PERF_RESULT plugins20_bounded_native_import_response source_bytes={} iterations_per_sample={} sample_pairs={} order=alternating_baseline_first_even percentile_method=nearest_rank baseline_full_response_buffers=2 bounded_full_response_buffers=1 baseline_source_text_clone_bytes={} bounded_source_text_clone_bytes=0 baseline_intermediate_metadata_bytes={} bounded_intermediate_metadata_bytes=0 baseline_p95_ns={} bounded_p95_ns={} ratio_bps={} threshold_bps={} baseline_samples_ns={} bounded_samples_ns={}",
        source.len(),
        BENCHMARK_ITERATIONS,
        BENCHMARK_SAMPLE_PAIRS,
        baseline_source_text_clone_bytes,
        baseline_intermediate_metadata_bytes,
        baseline_p95_ns,
        bounded_p95_ns,
        ratio_bps,
        BENCHMARK_TIME_RATIO_THRESHOLD_BPS,
        sample_csv(&baseline_raw),
        sample_csv(&bounded_raw),
    );

    assert_eq!(BENCHMARK_SAMPLE_PAIRS, baseline_raw.len());
    assert_eq!(BENCHMARK_SAMPLE_PAIRS, bounded_raw.len());
    assert!(
        ratio_bps <= BENCHMARK_TIME_RATIO_THRESHOLD_BPS,
        "bounded encoder P95 regression: ratio_bps={ratio_bps}"
    );
}

#[test]
fn packaged_native_manifest_uses_the_checked_in_generated_snapshot() {
    let rendered_manifest = PLUGIN_MANIFEST
        .strip_suffix('\0')
        .expect("native manifest must retain its ABI C-string terminator");

    assert_eq!(rendered_manifest, include_str!("../../plugin.toml"));
    assert!(rendered_manifest.starts_with("# @generated from Rust PluginDeclaration"));
    assert!(rendered_manifest.contains(&format!(
        "id = \"{}\"",
        NATIVE_DYNAMIC_FIXTURE_DECLARATION.id()
    )));
}

#[test]
fn declaration_projects_combined_runtime_and_editor_native_metadata() {
    assert_eq!(NATIVE_PLUGIN_ID, b"native_dynamic_fixture\0");
    assert_eq!(
        NATIVE_RUNTIME_ENTRY.cstr(),
        b"zircon_native_dynamic_fixture_runtime_entry_v3\0"
    );
    assert_eq!(
        NATIVE_EDITOR_ENTRY.cstr(),
        b"zircon_native_dynamic_fixture_editor_entry_v3\0"
    );
    assert_eq!(
        NATIVE_REQUESTED_CAPABILITIES,
        concat!(
            "runtime.plugin.native_dynamic_fixture\n",
            "runtime.asset.importer.native_dynamic_fixture.data_json\n",
            "editor.extension.native_dynamic_fixture\0",
        )
        .as_bytes()
    );

    let runtime_manifest = CStr::from_bytes_with_nul(NATIVE_RUNTIME_REGISTRATION_MANIFEST)
        .expect("runtime registration manifest is a C string")
        .to_str()
        .expect("runtime registration manifest is UTF-8");
    assert!(runtime_manifest.contains("[[systems]]"));
    assert!(runtime_manifest.contains("[[events]]"));
    assert!(runtime_manifest.contains("[[extensions]]"));
    assert!(!runtime_manifest.contains("editor.extension.native_dynamic_fixture"));

    let editor_manifest = CStr::from_bytes_with_nul(NATIVE_EDITOR_REGISTRATION_MANIFEST)
        .expect("editor registration manifest is a C string")
        .to_str()
        .expect("editor registration manifest is UTF-8");
    assert!(editor_manifest.contains("editor.extension.native_dynamic_fixture"));
    assert!(!editor_manifest.contains("runtime.plugin.native_dynamic_fixture"));
}

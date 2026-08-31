use std::hint::black_box;
use std::time::Instant;

use super::*;

const PATH_BYTES: usize = 64 * 1024;
const OPERATIONS_PER_SAMPLE: usize = 128;
const SAMPLE_PAIRS: usize = 21;

#[test]
fn optimization_batch_20260826hf_editor198_preserves_localization_projection() {
    let editor = map_localization_diagnostic(UiLocalizationDiagnostic::new(
        "empty_localized_text_key",
        UiLocalizationDiagnosticSeverity::Warning,
        "nodes.status.text.key",
        "localized text key is empty",
    ));
    assert_eq!(editor.code, LOCALIZATION_INVALID_REF_CODE);
    assert_eq!(editor.severity, UiAssetEditorDiagnosticSeverity::Warning);
    assert_eq!(editor.message, "localized text key is empty");
    assert_eq!(editor.source_path, "nodes.status.text.key");
    assert_eq!(editor.target_node_id.as_deref(), Some("status"));

    let editor = map_localization_diagnostic(UiLocalizationDiagnostic::new(
        "missing_message",
        UiLocalizationDiagnosticSeverity::Error,
        "asset.title",
        "message is missing",
    ));
    assert_eq!(editor.code, "missing_message");
    assert_eq!(editor.target_node_id, None);
}

#[test]
fn optimization_batch_20260826hf_editor198_moves_localization_source_path() {
    let source = include_str!("../localization.rs");
    let start = source
        .find("pub fn map_localization_diagnostic(")
        .expect("map_localization_diagnostic function");
    let end = source[start..]
        .find("\nfn editor_localization_code")
        .map(|offset| start + offset)
        .expect("next function boundary");
    let body = &source[start..end];

    assert!(body.contains("node_id_from_localization_path(&diagnostic.path)"));
    assert!(body.contains("diagnostic.path,"));
    assert!(!body.contains("source_path.clone()"));
}

#[test]
#[ignore = "managed release performance evidence"]
fn optimization_batch_20260826hf_editor198_localization_path_move_release_benchmark() {
    let diagnostic = benchmark_diagnostic();
    assert_eq!(
        map_localization_diagnostic(diagnostic.clone()),
        legacy_map_localization_diagnostic(diagnostic.clone())
    );

    let mut legacy_ns = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_ns = Vec::with_capacity(SAMPLE_PAIRS);
    for sample_index in 0..SAMPLE_PAIRS {
        let mut measure_legacy = || {
            let started = Instant::now();
            for _ in 0..OPERATIONS_PER_SAMPLE {
                black_box(legacy_map_localization_diagnostic(black_box(
                    diagnostic.clone(),
                )));
            }
            legacy_ns.push(started.elapsed().as_nanos().max(1));
        };
        let mut measure_optimized = || {
            let started = Instant::now();
            for _ in 0..OPERATIONS_PER_SAMPLE {
                black_box(map_localization_diagnostic(black_box(diagnostic.clone())));
            }
            optimized_ns.push(started.elapsed().as_nanos().max(1));
        };
        if sample_index % 2 == 0 {
            measure_legacy();
            measure_optimized();
        } else {
            measure_optimized();
            measure_legacy();
        }
    }

    let legacy_p50_ns = percentile(&legacy_ns, 50);
    let legacy_p95_ns = percentile(&legacy_ns, 95);
    let optimized_p50_ns = percentile(&optimized_ns, 50);
    let optimized_p95_ns = percentile(&optimized_ns, 95);
    println!(
        "EDITOR198_LOCALIZATION_PATH_MOVE_BENCH_V1 \
         path_bytes={PATH_BYTES} operations_per_sample={OPERATIONS_PER_SAMPLE} \
         sample_pairs={SAMPLE_PAIRS} legacy_p50_ns={legacy_p50_ns} \
         legacy_p95_ns={legacy_p95_ns} optimized_p50_ns={optimized_p50_ns} \
         optimized_p95_ns={optimized_p95_ns} legacy_ns={} optimized_ns={}",
        samples(&legacy_ns),
        samples(&optimized_ns),
    );
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70),
        "optimized P95 {optimized_p95_ns}ns must be at most 70% of legacy P95 {legacy_p95_ns}ns"
    );
}

fn benchmark_diagnostic() -> UiLocalizationDiagnostic {
    let path = format!(
        "nodes.status.{}",
        "translation_key.".repeat(PATH_BYTES / 16)
    );
    UiLocalizationDiagnostic::new(
        "missing_message",
        UiLocalizationDiagnosticSeverity::Warning,
        path,
        "localized message is missing",
    )
}

fn legacy_map_localization_diagnostic(
    diagnostic: UiLocalizationDiagnostic,
) -> UiAssetEditorDiagnostic {
    let code = editor_localization_code(&diagnostic).to_string();
    let source_path = diagnostic.path;
    let mut editor = UiAssetEditorDiagnostic::new(
        code,
        map_localization_severity(diagnostic.severity),
        diagnostic.message,
        source_path.clone(),
    );
    editor.target_node_id = node_id_from_localization_path(&source_path);
    editor
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut ordered = samples.to_vec();
    ordered.sort_unstable();
    let rank = ordered.len().saturating_mul(percentile).div_ceil(100);
    ordered[rank.saturating_sub(1)]
}

fn samples(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}

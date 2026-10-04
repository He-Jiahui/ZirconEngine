use std::hint::black_box;
use std::time::Instant;

use super::*;

const SAMPLE_PAIRS: usize = 21;

#[test]
fn import_pipeline_hotpath_metadata_partition_matches_clone_reference() {
    let diagnostics = vec![
        metadata_diagnostic(TextureMetadataDiagnosticSeverity::Warning, "warning-a"),
        metadata_diagnostic(TextureMetadataDiagnosticSeverity::Error, "error-a"),
        metadata_diagnostic(TextureMetadataDiagnosticSeverity::Warning, "warning-b"),
        metadata_diagnostic(TextureMetadataDiagnosticSeverity::Error, "error-b"),
    ];

    let legacy = legacy_partition_metadata_diagnostics(diagnostics.clone());
    let optimized = partition_metadata_diagnostics(diagnostics);

    assert_eq!(optimized, legacy);
    assert_eq!(
        optimized.0,
        vec!["error-a".to_string(), "error-b".to_string()]
    );
    assert_eq!(
        optimized.1,
        vec!["warning-a".to_string(), "warning-b".to_string()]
    );
}

#[test]
#[ignore = "release performance gate"]
fn import_pipeline_hotpath_metadata_partition_release_benchmark() {
    const DIAGNOSTIC_COUNT: usize = 8_192;
    const MESSAGE_BYTES: usize = 96;
    const REQUIRED_IMPROVEMENT_PERCENT: u128 = 40;

    let message = "m".repeat(MESSAGE_BYTES);
    let diagnostics = (0..DIAGNOSTIC_COUNT)
        .map(|_| metadata_diagnostic(TextureMetadataDiagnosticSeverity::Warning, &message))
        .collect::<Vec<_>>();
    assert_eq!(
        partition_metadata_diagnostics(diagnostics.clone()),
        legacy_partition_metadata_diagnostics(diagnostics.clone())
    );

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure_partition(
                &diagnostics,
                legacy_partition_metadata_diagnostics,
            ));
            optimized_samples.push(measure_partition(
                &diagnostics,
                partition_metadata_diagnostics,
            ));
        } else {
            optimized_samples.push(measure_partition(
                &diagnostics,
                partition_metadata_diagnostics,
            ));
            legacy_samples.push(measure_partition(
                &diagnostics,
                legacy_partition_metadata_diagnostics,
            ));
        }
    }

    let legacy_p95 = nearest_rank_p95(&legacy_samples);
    let optimized_p95 = nearest_rank_p95(&optimized_samples);
    let improvement = improvement_percent(legacy_p95, optimized_p95);
    println!(
        "PERF_RESULT plugins07_metadata_diagnostic_partition sample_pairs={} order=alternating_legacy_first_even diagnostics_per_sample={} message_bytes={} legacy_message_clones_per_sample={} optimized_message_clones_per_sample=0 legacy_ns={} optimized_ns={} legacy_p95_ns={} optimized_p95_ns={} threshold_percent={} improvement_percent={}",
        SAMPLE_PAIRS,
        DIAGNOSTIC_COUNT,
        MESSAGE_BYTES,
        DIAGNOSTIC_COUNT,
        samples_csv(&legacy_samples),
        samples_csv(&optimized_samples),
        legacy_p95,
        optimized_p95,
        REQUIRED_IMPROVEMENT_PERCENT,
        improvement
    );
    assert!(
        improvement >= REQUIRED_IMPROVEMENT_PERCENT,
        "single-pass diagnostic partition improved {improvement}%, below {REQUIRED_IMPROVEMENT_PERCENT}%"
    );
}

fn metadata_diagnostic(
    severity: TextureMetadataDiagnosticSeverity,
    message: &str,
) -> TextureMetadataDiagnostic {
    TextureMetadataDiagnostic {
        severity,
        message: message.to_string(),
    }
}

fn legacy_partition_metadata_diagnostics(
    diagnostics: Vec<TextureMetadataDiagnostic>,
) -> (Vec<String>, Vec<String>) {
    let diagnostics = diagnostics.into_iter();
    let errors = diagnostics
        .clone()
        .filter(|diagnostic| diagnostic.severity == TextureMetadataDiagnosticSeverity::Error)
        .map(|diagnostic| diagnostic.message)
        .collect::<Vec<_>>();
    let warnings = diagnostics
        .filter(|diagnostic| diagnostic.severity == TextureMetadataDiagnosticSeverity::Warning)
        .map(|diagnostic| diagnostic.message)
        .collect::<Vec<_>>();
    (errors, warnings)
}

fn measure_partition(
    diagnostics: &[TextureMetadataDiagnostic],
    partition: fn(Vec<TextureMetadataDiagnostic>) -> (Vec<String>, Vec<String>),
) -> u128 {
    let owned = diagnostics.to_vec();
    let started = Instant::now();
    black_box(partition(black_box(owned)));
    started.elapsed().as_nanos()
}

fn nearest_rank_p95(samples: &[u128]) -> u128 {
    assert_eq!(samples.len(), SAMPLE_PAIRS);
    let mut ordered = samples.to_vec();
    ordered.sort_unstable();
    ordered[(ordered.len() * 95).div_ceil(100) - 1]
}

fn improvement_percent(legacy: u128, optimized: u128) -> u128 {
    assert!(legacy > 0);
    legacy.saturating_sub(optimized) * 100 / legacy
}

fn samples_csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}

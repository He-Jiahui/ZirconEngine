use std::hint::black_box;
use std::time::Instant;

use super::{
    TextureMetadataDiagnostic, TextureMetadataDiagnosticSeverity,
    format_cached_texture_metadata_errors,
};

const SAMPLE_PAIRS: usize = 21;
const FORMATS_PER_SAMPLE: usize = 32_768;
const URI: &str = "res://textures/environment/production_sky.ztexture";

#[test]
fn optimization_batch_20260826dq_runtime160_cache_texture_metadata_preserves_error_text() {
    let diagnostics = vec![
        diagnostic(
            TextureMetadataDiagnosticSeverity::Warning,
            "ignored warning",
        ),
        diagnostic(
            TextureMetadataDiagnosticSeverity::Error,
            "invalid mip count",
        ),
        diagnostic(
            TextureMetadataDiagnosticSeverity::Error,
            "unsupported compression family",
        ),
    ];
    assert_eq!(
        format_cached_texture_metadata_errors(URI, &diagnostics).as_deref(),
        Some(
            "validate cached texture metadata res://textures/environment/production_sky.ztexture: invalid mip count; unsupported compression family"
        )
    );
    assert_eq!(
        format_cached_texture_metadata_errors(
            URI,
            &[diagnostic(
                TextureMetadataDiagnosticSeverity::Warning,
                "warning only",
            )],
        ),
        None
    );
}

#[test]
fn optimization_batch_20260826dq_runtime160_cache_texture_metadata_writes_one_exact_buffer() {
    let diagnostics = fixture_diagnostics();
    let message = format_cached_texture_metadata_errors(URI, &diagnostics).unwrap();
    assert_eq!(message.capacity(), message.len());

    let source = include_str!("../cache_payload.rs");
    assert!(source.contains("let message_capacity = PREFIX.len()"));
    assert!(source.contains("let mut message = String::with_capacity(message_capacity);"));
    assert!(source.contains("format_cached_texture_metadata_errors(&uri, &diagnostics)"));
    assert!(!source.contains("errors.join(\"; \")"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826dq_runtime160_cache_texture_metadata_single_buffer_bench() {
    let diagnostics = fixture_diagnostics();
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);

    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(&diagnostics, legacy_format));
            optimized_samples.push(measure(&diagnostics, format_cached_texture_metadata_errors));
        } else {
            optimized_samples.push(measure(&diagnostics, format_cached_texture_metadata_errors));
            legacy_samples.push(measure(&diagnostics, legacy_format));
        }
    }

    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "RUNTIME160_CACHE_TEXTURE_METADATA_SINGLE_BUFFER_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
formats_per_sample={FORMATS_PER_SAMPLE} legacy_allocations_per_format=3 \
optimized_allocations_per_format=1 legacy_p50_ns={legacy_p50_ns} \
optimized_p50_ns={optimized_p50_ns} legacy_p95_ns={legacy_p95_ns} \
optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70),
        "single-buffer cache metadata formatting P95 {optimized_p95_ns}ns must be at most 70% of collect/join formatting P95 {legacy_p95_ns}ns"
    );
}

fn diagnostic(
    severity: TextureMetadataDiagnosticSeverity,
    message: &str,
) -> TextureMetadataDiagnostic {
    TextureMetadataDiagnostic {
        severity,
        message: message.to_string(),
    }
}

fn fixture_diagnostics() -> Vec<TextureMetadataDiagnostic> {
    (0..12)
        .map(|index| {
            diagnostic(
                if index % 5 == 0 {
                    TextureMetadataDiagnosticSeverity::Warning
                } else {
                    TextureMetadataDiagnosticSeverity::Error
                },
                "texture metadata violation requires conversion before cached upload",
            )
        })
        .collect()
}

fn legacy_format(uri: &str, diagnostics: &[TextureMetadataDiagnostic]) -> Option<String> {
    let errors = diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.severity == TextureMetadataDiagnosticSeverity::Error)
        .map(|diagnostic| diagnostic.message.as_str())
        .collect::<Vec<_>>();
    if errors.is_empty() {
        None
    } else {
        Some(format!(
            "validate cached texture metadata {uri}: {}",
            errors.join("; ")
        ))
    }
}

fn measure(
    diagnostics: &[TextureMetadataDiagnostic],
    render: fn(&str, &[TextureMetadataDiagnostic]) -> Option<String>,
) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..FORMATS_PER_SAMPLE {
        checksum ^= black_box(render(black_box(URI), black_box(diagnostics)))
            .unwrap()
            .len();
    }
    black_box(checksum);
    started.elapsed().as_nanos().max(1)
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn sample_csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}

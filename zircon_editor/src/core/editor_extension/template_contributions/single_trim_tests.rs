use std::hint::black_box;
use std::time::Instant;

use super::super::validate_ui_template_document;

const DOCUMENT_BYTES: usize = 4096;
const DOCUMENT_COUNT: usize = 4096;
const SAMPLE_PAIRS: usize = 31;

fn legacy_validate(document: &str) -> bool {
    !(document.trim().is_empty() || document.trim() != document || !document.ends_with(".zui"))
}

fn fixture_documents() -> Vec<String> {
    (0..DOCUMENT_COUNT)
        .map(|index| {
            let suffix = format!(".pane_{index}.zui");
            format!("{}{}", "a".repeat(DOCUMENT_BYTES - suffix.len()), suffix)
        })
        .collect()
}

fn measure(documents: &[String], optimized: bool) -> u128 {
    let started = Instant::now();
    let valid = documents
        .iter()
        .filter(|document| {
            if optimized {
                validate_ui_template_document("ui template document", black_box(document.as_str()))
                    .is_ok()
            } else {
                legacy_validate(black_box(document.as_str()))
            }
        })
        .count();
    black_box(valid);
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

#[test]
fn optimization_batch_20260829az_editor272_single_trim_validation_preserves_rules() {
    for document in [
        "pane.zui",
        "nested/pane.zui",
        "",
        "   ",
        " pane.zui",
        "pane.zui ",
        "\u{2003}pane.zui",
        "pane.toml",
    ] {
        assert_eq!(
            validate_ui_template_document("ui template document", document).is_ok(),
            legacy_validate(document),
            "{document:?}"
        );
    }
}

#[test]
fn optimization_batch_20260829az_editor272_template_validation_uses_one_trim() {
    let source = include_str!("../template_contributions.rs");
    let production = source.split_once("#[cfg(test)]").expect("test boundary").0;

    assert_eq!(production.matches("document.trim()").count(), 1);
    assert!(production.contains("trimmed.len() != document.len()"));
    assert!(!production.contains("document.trim() != document"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260829az_editor272_single_trim_ui_template_validation_bench() {
    let documents = fixture_documents();
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(&documents, false));
            optimized_samples.push(measure(&documents, true));
        } else {
            optimized_samples.push(measure(&documents, true));
            legacy_samples.push(measure(&documents, false));
        }
    }

    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "EDITOR272_SINGLE_TRIM_UI_TEMPLATE_VALIDATION_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
documents={DOCUMENT_COUNT} bytes_per_document={DOCUMENT_BYTES} legacy_trim_calls=2 optimized_trim_calls=1 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

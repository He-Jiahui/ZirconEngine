use std::hint::black_box;
use std::time::Instant;

use super::super::DocumentKind;

const SAMPLE_PAIRS: usize = 31;
const VALUE_COUNT: usize = 8192;

fn sample_values() -> Vec<String> {
    (0..VALUE_COUNT)
        .map(|index| {
            let prefix = format!(
                "scene.layer_{index}.animation_sequence.material_template.prefab_tree.resource_binding.property_path"
            );
            match index % 8 {
                0 => prefix,
                1 => format!("{prefix}.leaf-{index}"),
                2 => format!("{prefix}..invalid"),
                3 => format!("{prefix}.Invalid"),
                4 => format!("{prefix}.invalid/leaf"),
                5 => format!("{prefix}."),
                6 => format!(".{prefix}"),
                _ => format!("{prefix}.leaf_{index}"),
            }
        })
        .collect()
}

fn legacy_valid(value: &str) -> bool {
    !value.is_empty()
        && value.split('.').all(|segment| {
            !segment.is_empty()
                && segment.chars().all(|character| {
                    character.is_ascii_lowercase()
                        || character.is_ascii_digit()
                        || character == '_'
                        || character == '-'
                })
        })
}

fn legacy_parse(value: &str) -> Result<String, String> {
    let value = value.to_owned();
    if legacy_valid(&value) {
        Ok(value)
    } else {
        Err(value)
    }
}

fn measure(values: &[String], optimized: bool) -> u128 {
    let started = Instant::now();
    let valid = values
        .iter()
        .filter(|value| {
            if optimized {
                DocumentKind::parse(black_box(value.as_str())).is_ok()
            } else {
                legacy_parse(black_box(value.as_str())).is_ok()
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
fn optimization_batch_20260829ay_editor271_byte_scanned_document_kinds_preserve_segment_validation()
{
    for value in [
        "scene",
        "scene.layer_1",
        "animation-sequence.clip",
        "",
        ".scene",
        "scene.",
        "scene..layer",
        "Scene",
        "scene/leaf",
        "scene.\u{4f8b}",
    ] {
        assert_eq!(
            DocumentKind::parse(value).is_ok(),
            legacy_valid(value),
            "{value:?}"
        );
    }
}

#[test]
fn optimization_batch_20260829ay_editor271_byte_scanned_document_kinds_keep_the_original_owned_value_on_error(
) {
    let value = String::from("scene..invalid");
    let error = DocumentKind::parse(value.clone()).expect_err("invalid kind");
    assert_eq!(
        error.to_string(),
        "editor document kind `scene..invalid` is invalid"
    );
}

#[test]
#[ignore = "release performance evidence for the managed validation coordinator"]
fn optimization_batch_20260829ay_editor271_byte_scan_document_kind_bench() {
    let values = sample_values();
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(&values, false));
            optimized_samples.push(measure(&values, true));
        } else {
            optimized_samples.push(measure(&values, true));
            legacy_samples.push(measure(&values, false));
        }
    }

    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "EDITOR271_BYTE_SCAN_DOCUMENT_KIND_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
values={VALUE_COUNT} legacy_string_allocations={VALUE_COUNT} optimized_string_allocations={VALUE_COUNT} \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

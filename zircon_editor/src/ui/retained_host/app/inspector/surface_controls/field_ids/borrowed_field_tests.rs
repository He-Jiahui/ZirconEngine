use std::borrow::Cow;
use std::hint::black_box;
use std::time::Instant;

use super::inspector_field_id;

const CHECKS_PER_SAMPLE: usize = 131_072;
const FIELD_BYTES: usize = 512;
const SAMPLE_PAIRS: usize = 31;

fn legacy_inspector_field_id(control_id: &str) -> Option<String> {
    if let Some(field_id) = control_id.strip_prefix("DynamicComponentField:") {
        return Some(field_id.to_string());
    }
    match control_id {
        "NameField" => Some("name".to_string()),
        "ParentField" => Some("parent".to_string()),
        "PositionXField" => Some("transform.translation.x".to_string()),
        "PositionYField" => Some("transform.translation.y".to_string()),
        "PositionZField" => Some("transform.translation.z".to_string()),
        _ => None,
    }
}

fn measure(control_id: &str, optimized: bool) -> u128 {
    let started = Instant::now();
    let mut bytes = 0;
    for _ in 0..CHECKS_PER_SAMPLE {
        bytes += if optimized {
            inspector_field_id(black_box(control_id)).map_or(0, |field| field.len())
        } else {
            legacy_inspector_field_id(black_box(control_id)).map_or(0, |field| field.len())
        };
    }
    black_box(bytes);
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
fn optimization_batch_20260829bd_editor276_borrowed_field_ids_preserve_values() {
    for control_id in [
        "DynamicComponentField:transform.scale.x",
        "NameField",
        "ParentField",
        "PositionXField",
        "PositionYField",
        "PositionZField",
        "UnknownField",
    ] {
        assert_eq!(
            inspector_field_id(control_id).as_deref(),
            legacy_inspector_field_id(control_id).as_deref(),
            "{control_id:?}"
        );
    }
}

#[test]
fn optimization_batch_20260829bd_editor276_inspector_field_ids_are_borrowed() {
    assert!(matches!(
        inspector_field_id("DynamicComponentField:material.roughness"),
        Some(Cow::Borrowed("material.roughness"))
    ));
    assert!(matches!(
        inspector_field_id("NameField"),
        Some(Cow::Borrowed("name"))
    ));

    let source = include_str!("../field_ids.rs");
    let production = source.split_once("#[cfg(test)]").expect("test boundary").0;
    assert!(!production.contains(".to_string()"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260829bd_editor276_borrowed_inspector_field_ids_bench() {
    let prefix = "DynamicComponentField:";
    let control_id = format!("{prefix}{}", "f".repeat(FIELD_BYTES - prefix.len()));
    assert_eq!(control_id.len(), FIELD_BYTES);
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(&control_id, false));
            optimized_samples.push(measure(&control_id, true));
        } else {
            optimized_samples.push(measure(&control_id, true));
            legacy_samples.push(measure(&control_id, false));
        }
    }

    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "EDITOR276_BORROWED_INSPECTOR_FIELD_IDS_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
checks_per_sample={CHECKS_PER_SAMPLE} field_bytes={FIELD_BYTES} \
legacy_parser_allocations_per_check=1 optimized_parser_allocations_per_check=0 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

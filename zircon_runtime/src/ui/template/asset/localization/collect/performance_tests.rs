use std::hint::black_box;
use std::time::Instant;

use toml::Value;
use zircon_runtime_interface::ui::template::{
    UiLocalizationDependency, UiLocalizationDiagnostic, UiLocalizationDiagnosticSeverity,
    UiLocalizationReport, UiLocalizationTextCandidate,
};

use super::*;

const VALUE_GROUPS: usize = 100;
const VALUES_PER_GROUP: usize = 100;
const SAMPLE_PAIRS: usize = 21;

#[test]
fn buffered_document_value_paths_match_legacy_collection() {
    let value = value_fixture(8, 8);
    let mut legacy = UiLocalizationReport::default();
    legacy_collect_value("nodes.root.props", &value, &mut legacy);
    let mut optimized = UiLocalizationReport::default();
    collect_value(&mut "nodes.root.props".to_string(), &value, &mut optimized);
    assert_eq!(optimized, legacy);
}

#[test]
#[ignore = "release-only document localization path benchmark"]
fn document_localization_path_buffer_release_benchmark_evidence() {
    let value = value_fixture(VALUE_GROUPS, VALUES_PER_GROUP);
    black_box(time_legacy(&value));
    black_box(time_buffered(&value));

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(time_legacy(&value));
            optimized_samples.push(time_buffered(&value));
        } else {
            optimized_samples.push(time_buffered(&value));
            legacy_samples.push(time_legacy(&value));
        }
    }

    let legacy_p95_ns = nearest_rank(&legacy_samples, 95);
    let optimized_p95_ns = nearest_rank(&optimized_samples, 95);
    let leaf_count = VALUE_GROUPS * VALUES_PER_GROUP;
    let legacy_temporary_path_allocations = VALUE_GROUPS + leaf_count * 2;

    println!(
        "RUNTIME83_LOCALIZATION_COLLECT_PERF leaf_count={} pairs={} order=alternating percentile=nearest-rank legacy_temporary_path_allocations={} optimized_temporary_path_allocations=0 retained_path_clones={} legacy_p50_ns={} legacy_p95_ns={} optimized_p50_ns={} optimized_p95_ns={} legacy_samples_ns={:?} optimized_samples_ns={:?}",
        leaf_count,
        SAMPLE_PAIRS,
        legacy_temporary_path_allocations,
        leaf_count,
        nearest_rank(&legacy_samples, 50),
        legacy_p95_ns,
        nearest_rank(&optimized_samples, 50),
        optimized_p95_ns,
        legacy_samples,
        optimized_samples,
    );

    assert_eq!(legacy_temporary_path_allocations, 20_100);
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(90),
        "buffered document localization paths must reduce P95 by at least 10%: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );
}

fn value_fixture(groups: usize, values_per_group: usize) -> Value {
    let mut root = toml::map::Map::new();
    for group in 0..groups {
        let mut entries = toml::map::Map::new();
        for value in 0..values_per_group {
            let mut text = toml::map::Map::new();
            text.insert("text".to_string(), Value::String("localized".to_string()));
            entries.insert(format!("item-{value:05}"), Value::Table(text));
        }
        root.insert(format!("group-{group:05}"), Value::Table(entries));
    }
    Value::Table(root)
}

fn legacy_collect_value(path: &str, value: &Value, report: &mut UiLocalizationReport) {
    match value {
        Value::String(text) if is_text_path(path) => {
            report
                .extraction_candidates
                .push(UiLocalizationTextCandidate {
                    path: path.to_string(),
                    text: text.clone(),
                });
        }
        Value::Table(table) => {
            if let Some(reference) = localized_text_ref(table) {
                if let Some(message) = reference.validate(path) {
                    report.diagnostics.push(UiLocalizationDiagnostic::new(
                        "empty_localized_text_key",
                        UiLocalizationDiagnosticSeverity::Error,
                        path,
                        message,
                    ));
                    return;
                }
                report.dependencies.push(UiLocalizationDependency {
                    path: path.to_string(),
                    reference,
                    direction: text_direction(table),
                });
                return;
            }
            for (key, nested) in table {
                legacy_collect_value(&format!("{path}.{key}"), nested, report);
            }
        }
        Value::Array(items) => {
            for (index, item) in items.iter().enumerate() {
                legacy_collect_value(&format!("{path}[{index}]"), item, report);
            }
        }
        _ => {}
    }
}

fn time_legacy(value: &Value) -> u128 {
    let started = Instant::now();
    let mut report = UiLocalizationReport::default();
    legacy_collect_value("nodes.root.props", black_box(value), &mut report);
    let elapsed = started.elapsed().as_nanos();
    black_box(report);
    elapsed
}

fn time_buffered(value: &Value) -> u128 {
    let started = Instant::now();
    let mut report = UiLocalizationReport::default();
    collect_value(
        &mut "nodes.root.props".to_string(),
        black_box(value),
        &mut report,
    );
    let elapsed = started.elapsed().as_nanos();
    black_box(report);
    elapsed
}

fn nearest_rank(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

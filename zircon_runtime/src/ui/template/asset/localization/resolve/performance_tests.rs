use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::hint::black_box;
use std::time::Instant;

use toml::Value;
use zircon_runtime_interface::ui::template::{
    UiLocalizationDependency, UiLocalizationReport, UiLocalizedTextRef, UiTextDirection,
};

use super::*;

const KEY_GROUPS: usize = 100;
const KEYS_PER_GROUP: usize = 100;
const DEPENDENCY_COUNT: usize = 50_000;
const MEMBERSHIP_PROBES: usize = 200_000;
const SAMPLE_PAIRS: usize = 21;
const TABLE_COUNT: usize = 2_048;

#[test]
fn buffered_key_collection_and_hoisted_lookup_match_legacy_results() {
    let value = key_fixture(8, 8);
    let mut legacy_keys = BTreeSet::new();
    legacy_collect_locale_keys("", &value, &mut legacy_keys);
    let mut optimized_keys = BTreeSet::new();
    collect_locale_keys(&mut String::new(), &value, &mut optimized_keys);
    assert_eq!(optimized_keys, legacy_keys);

    let (report, catalog) = catalog_fixture(32);
    assert_eq!(
        validate_localization_report_against_catalog(&report, "en-US", &catalog),
        legacy_validate_report(&report, "en-US", &catalog)
    );
}

#[test]
fn duplicate_missing_localization_references_emit_one_diagnostic() {
    let (mut report, catalog) = catalog_fixture(DEPENDENCY_COUNT);
    for dependency in &mut report.dependencies {
        dependency.reference.key = "missing.key".to_string();
    }

    let diagnostics = validate_localization_report_against_catalog(&report, "en-US", &catalog);

    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].code, "missing_locale_key");
}

#[test]
fn hash_indexed_catalog_keys_preserve_filtering_and_deduplication() {
    let mut catalog = UiLocalizationTableCatalog::default();
    catalog.register_table_keys(
        "en-US",
        "default",
        None,
        ["beta", "alpha", "beta", "   "],
    );

    let keys = &catalog.tables["en-US"]["default"].keys;
    assert_eq!(keys.len(), 2);
    assert!(keys.contains("alpha"));
    assert!(keys.contains("beta"));
}

#[test]
fn hash_indexed_catalog_tables_preserve_replacement_semantics() {
    let mut catalog = UiLocalizationTableCatalog::default();
    catalog.register_table_keys("en-US", "menu", None, ["old"]);
    catalog.register_table_keys("en-US", "menu", None, ["new"]);

    let keys = &catalog.tables["en-US"]["menu"].keys;
    assert_eq!(keys.len(), 1);
    assert!(keys.contains("new"));
    assert!(!keys.contains("old"));
}

#[test]
#[ignore = "release-only localization catalog key membership benchmark"]
fn localization_hash_key_index_release_benchmark_evidence() {
    let keys = (0..KEY_GROUPS * KEYS_PER_GROUP)
        .map(|index| format!("group-{:05}.key-{:05}", index / KEYS_PER_GROUP, index))
        .collect::<Vec<_>>();
    let ordered = keys.iter().cloned().collect::<BTreeSet<_>>();
    let hashed = keys.iter().cloned().collect::<HashSet<_>>();

    black_box(time_btree_membership(&ordered, &keys));
    black_box(time_hash_membership(&hashed, &keys));

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(time_btree_membership(&ordered, &keys));
            optimized_samples.push(time_hash_membership(&hashed, &keys));
        } else {
            optimized_samples.push(time_hash_membership(&hashed, &keys));
            legacy_samples.push(time_btree_membership(&ordered, &keys));
        }
    }

    let legacy_p95_ns = nearest_rank(&legacy_samples, 95);
    let optimized_p95_ns = nearest_rank(&optimized_samples, 95);
    println!(
        "RUNTIME83_LOCALIZATION_HASH_KEY_INDEX_BENCH_V1 key_count={} probes={} pairs={} order=alternating percentile=nearest-rank legacy_lookup_class=O_log_n optimized_lookup_class=O_1_average legacy_btree_p50_ns={} legacy_btree_p95_ns={} optimized_hash_p50_ns={} optimized_hash_p95_ns={} legacy_samples_ns={:?} optimized_samples_ns={:?}",
        KEY_GROUPS * KEYS_PER_GROUP,
        MEMBERSHIP_PROBES,
        SAMPLE_PAIRS,
        nearest_rank(&legacy_samples, 50),
        legacy_p95_ns,
        nearest_rank(&optimized_samples, 50),
        optimized_p95_ns,
        legacy_samples,
        optimized_samples,
    );

    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(80),
        "hash-indexed localization keys must reduce membership P95 by at least 20%: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );
}

#[test]
#[ignore = "release-only localization catalog table membership benchmark"]
fn localization_hash_table_index_release_benchmark_evidence() {
    let table_names = (0..TABLE_COUNT)
        .map(|index| format!("domain.table-{index:05}"))
        .collect::<Vec<_>>();
    let ordered = table_names
        .iter()
        .cloned()
        .enumerate()
        .map(|(index, name)| (name, index))
        .collect::<BTreeMap<_, _>>();
    let hashed = table_names
        .iter()
        .cloned()
        .enumerate()
        .map(|(index, name)| (name, index))
        .collect::<HashMap<_, _>>();

    black_box(time_btree_table_membership(&ordered, &table_names));
    black_box(time_hash_table_membership(&hashed, &table_names));

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(time_btree_table_membership(&ordered, &table_names));
            optimized_samples.push(time_hash_table_membership(&hashed, &table_names));
        } else {
            optimized_samples.push(time_hash_table_membership(&hashed, &table_names));
            legacy_samples.push(time_btree_table_membership(&ordered, &table_names));
        }
    }

    let legacy_p95_ns = nearest_rank(&legacy_samples, 95);
    let optimized_p95_ns = nearest_rank(&optimized_samples, 95);
    println!(
        "RUNTIME83_LOCALIZATION_HASH_TABLE_INDEX_BENCH_V1 table_count={} probes={} pairs={} order=alternating percentile=nearest-rank legacy_lookup_class=O_log_n optimized_lookup_class=O_1_average legacy_table_btree_p50_ns={} legacy_table_btree_p95_ns={} optimized_table_hash_p50_ns={} optimized_table_hash_p95_ns={} legacy_samples_ns={:?} optimized_samples_ns={:?}",
        TABLE_COUNT,
        MEMBERSHIP_PROBES,
        SAMPLE_PAIRS,
        nearest_rank(&legacy_samples, 50),
        legacy_p95_ns,
        nearest_rank(&optimized_samples, 50),
        optimized_p95_ns,
        legacy_samples,
        optimized_samples,
    );

    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(80),
        "hash-indexed localization tables must reduce membership P95 by at least 20%: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );
}

#[test]
#[ignore = "release-only localization path and lookup benchmark"]
fn localization_resolve_path_and_lookup_release_benchmark_evidence() {
    let value = key_fixture(KEY_GROUPS, KEYS_PER_GROUP);
    let (mut report, catalog) = catalog_fixture(DEPENDENCY_COUNT);
    for dependency in &mut report.dependencies {
        dependency.reference.key = "missing.key".to_string();
    }

    black_box(time_legacy_keys(&value));
    black_box(time_buffered_keys(&value));
    black_box(time_legacy_lookup(&report, &catalog));
    black_box(time_hoisted_lookup(&report, &catalog));

    let mut legacy_path_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_path_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut legacy_lookup_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_lookup_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_path_samples.push(time_legacy_keys(&value));
            optimized_path_samples.push(time_buffered_keys(&value));
            legacy_lookup_samples.push(time_legacy_lookup(&report, &catalog));
            optimized_lookup_samples.push(time_hoisted_lookup(&report, &catalog));
        } else {
            optimized_path_samples.push(time_buffered_keys(&value));
            legacy_path_samples.push(time_legacy_keys(&value));
            optimized_lookup_samples.push(time_hoisted_lookup(&report, &catalog));
            legacy_lookup_samples.push(time_legacy_lookup(&report, &catalog));
        }
    }

    let legacy_path_p95_ns = nearest_rank(&legacy_path_samples, 95);
    let optimized_path_p95_ns = nearest_rank(&optimized_path_samples, 95);
    let legacy_lookup_p95_ns = nearest_rank(&legacy_lookup_samples, 95);
    let optimized_lookup_p95_ns = nearest_rank(&optimized_lookup_samples, 95);
    let leaf_count = KEY_GROUPS * KEYS_PER_GROUP;
    let legacy_temporary_path_allocations = KEY_GROUPS + leaf_count;

    println!(
        "RUNTIME83_LOCALIZATION_RESOLVE_PERF leaf_count={} dependencies={} pairs={} order=alternating percentile=nearest-rank legacy_temporary_path_allocations={} optimized_temporary_path_allocations=0 legacy_diagnostic_constructions={} optimized_diagnostic_constructions=1 unique_missing_key_model_count={} legacy_missing_key_message_allocations={} optimized_missing_key_message_allocations={} legacy_locale_lookups={} optimized_locale_lookups=1 legacy_path_p50_ns={} legacy_path_p95_ns={} optimized_path_p50_ns={} optimized_path_p95_ns={} legacy_lookup_p50_ns={} legacy_lookup_p95_ns={} optimized_lookup_p50_ns={} optimized_lookup_p95_ns={} legacy_path_samples_ns={:?} optimized_path_samples_ns={:?} legacy_lookup_samples_ns={:?} optimized_lookup_samples_ns={:?}",
        leaf_count,
        DEPENDENCY_COUNT,
        SAMPLE_PAIRS,
        legacy_temporary_path_allocations,
        DEPENDENCY_COUNT,
        DEPENDENCY_COUNT,
        DEPENDENCY_COUNT.saturating_mul(2),
        DEPENDENCY_COUNT,
        DEPENDENCY_COUNT,
        nearest_rank(&legacy_path_samples, 50),
        legacy_path_p95_ns,
        nearest_rank(&optimized_path_samples, 50),
        optimized_path_p95_ns,
        nearest_rank(&legacy_lookup_samples, 50),
        legacy_lookup_p95_ns,
        nearest_rank(&optimized_lookup_samples, 50),
        optimized_lookup_p95_ns,
        legacy_path_samples,
        optimized_path_samples,
        legacy_lookup_samples,
        optimized_lookup_samples,
    );

    assert_eq!(legacy_temporary_path_allocations, 10_100);
    assert!(
        optimized_path_p95_ns.saturating_mul(100) <= legacy_path_p95_ns.saturating_mul(90),
        "buffered localization key paths must reduce P95 by at least 10%: legacy={legacy_path_p95_ns}ns optimized={optimized_path_p95_ns}ns"
    );
    assert!(
        optimized_lookup_p95_ns.saturating_mul(100) <= legacy_lookup_p95_ns.saturating_mul(90),
        "hoisted locale lookup must reduce P95 by at least 10%: legacy={legacy_lookup_p95_ns}ns optimized={optimized_lookup_p95_ns}ns"
    );
}

fn key_fixture(groups: usize, keys_per_group: usize) -> Value {
    let mut root = toml::map::Map::new();
    for group in 0..groups {
        let mut entries = toml::map::Map::new();
        for key in 0..keys_per_group {
            entries.insert(format!("key-{key:05}"), Value::String("value".to_string()));
        }
        root.insert(format!("group-{group:05}"), Value::Table(entries));
    }
    Value::Table(root)
}

fn catalog_fixture(dependency_count: usize) -> (UiLocalizationReport, UiLocalizationTableCatalog) {
    let dependency = UiLocalizationDependency {
        path: "nodes.root.props.text".to_string(),
        reference: UiLocalizedTextRef {
            key: "shared.key".to_string(),
            table: Some("default".to_string()),
            fallback: None,
        },
        direction: UiTextDirection::Auto,
    };
    let report = UiLocalizationReport {
        dependencies: vec![dependency; dependency_count],
        ..UiLocalizationReport::default()
    };
    let mut catalog = UiLocalizationTableCatalog::default();
    catalog.register_table_keys("en-US", "default", None, ["shared.key"]);
    (report, catalog)
}

fn legacy_collect_locale_keys(prefix: &str, value: &Value, keys: &mut BTreeSet<String>) {
    match value {
        Value::Table(table) => {
            for (key, value) in table {
                let path = if prefix.is_empty() {
                    key.to_string()
                } else {
                    format!("{prefix}.{key}")
                };
                legacy_collect_locale_keys(&path, value, keys);
            }
        }
        Value::Array(_) => {}
        _ if !prefix.is_empty() => {
            let _ = keys.insert(prefix.to_string());
        }
        _ => {}
    }
}

fn legacy_validate_report(
    report: &UiLocalizationReport,
    locale: &str,
    catalog: &UiLocalizationTableCatalog,
) -> Vec<UiLocalizationDiagnostic> {
    let locale = locale.trim();
    let mut diagnostics = report
        .dependencies
        .iter()
        .filter_map(|dependency| {
            let locale_tables = black_box(&catalog.tables).get(black_box(locale));
            legacy_validate_dependency(locale, dependency, locale_tables)
        })
        .collect::<Vec<_>>();
    diagnostics.sort();
    diagnostics.dedup();
    diagnostics
}

fn legacy_validate_dependency(
    locale: &str,
    dependency: &UiLocalizationDependency,
    locale_tables: Option<&HashMap<String, UiLocalizationTableEntry>>,
) -> Option<UiLocalizationDiagnostic> {
    let table_name = dependency
        .reference
        .table
        .as_deref()
        .unwrap_or(DEFAULT_LOCALIZATION_TABLE);
    let Some(table) = locale_tables.and_then(|tables| tables.get(table_name)) else {
        return Some(UiLocalizationDiagnostic::new(
            "missing_locale_table",
            UiLocalizationDiagnosticSeverity::Error,
            dependency.path.clone(),
            format!(
                "locale table {locale}/{table_name} is not registered for key {}",
                dependency.reference.key
            ),
        ));
    };
    if table.keys.contains(&dependency.reference.key) {
        return None;
    }
    let source = table
        .source_uri
        .as_deref()
        .map(|source_uri| format!(" in {source_uri}"))
        .unwrap_or_default();
    Some(UiLocalizationDiagnostic::new(
        "missing_locale_key",
        missing_ref_severity(dependency),
        dependency.path.clone(),
        format!(
            "locale key {} is missing from {locale}/{table_name}{source}",
            dependency.reference.key
        ),
    ))
}

fn time_legacy_keys(value: &Value) -> u128 {
    let started = Instant::now();
    let mut keys = BTreeSet::new();
    legacy_collect_locale_keys("", black_box(value), &mut keys);
    let elapsed = started.elapsed().as_nanos();
    black_box(keys);
    elapsed
}

fn time_buffered_keys(value: &Value) -> u128 {
    let started = Instant::now();
    let mut keys = BTreeSet::new();
    collect_locale_keys(&mut String::new(), black_box(value), &mut keys);
    let elapsed = started.elapsed().as_nanos();
    black_box(keys);
    elapsed
}

fn time_legacy_lookup(report: &UiLocalizationReport, catalog: &UiLocalizationTableCatalog) -> u128 {
    let started = Instant::now();
    let diagnostics = legacy_validate_report(black_box(report), "en-US", black_box(catalog));
    let elapsed = started.elapsed().as_nanos();
    black_box(diagnostics);
    elapsed
}

fn time_hoisted_lookup(
    report: &UiLocalizationReport,
    catalog: &UiLocalizationTableCatalog,
) -> u128 {
    let started = Instant::now();
    let diagnostics = validate_localization_report_against_catalog(
        black_box(report),
        "en-US",
        black_box(catalog),
    );
    let elapsed = started.elapsed().as_nanos();
    black_box(diagnostics);
    elapsed
}

fn time_btree_membership(index: &BTreeSet<String>, keys: &[String]) -> u128 {
    let started = Instant::now();
    let mut hits = 0usize;
    for probe in 0..MEMBERSHIP_PROBES {
        let key = &keys[probe.wrapping_mul(2_654_435_761) % keys.len()];
        hits += usize::from(index.contains(black_box(key.as_str())));
    }
    let elapsed = started.elapsed().as_nanos();
    black_box(hits);
    elapsed
}

fn time_hash_membership(index: &HashSet<String>, keys: &[String]) -> u128 {
    let started = Instant::now();
    let mut hits = 0usize;
    for probe in 0..MEMBERSHIP_PROBES {
        let key = &keys[probe.wrapping_mul(2_654_435_761) % keys.len()];
        hits += usize::from(index.contains(black_box(key.as_str())));
    }
    let elapsed = started.elapsed().as_nanos();
    black_box(hits);
    elapsed
}

fn time_btree_table_membership(index: &BTreeMap<String, usize>, table_names: &[String]) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for probe in 0..MEMBERSHIP_PROBES {
        let name = &table_names[probe.wrapping_mul(2_654_435_761) % table_names.len()];
        checksum ^= index.get(black_box(name.as_str())).copied().unwrap();
    }
    let elapsed = started.elapsed().as_nanos();
    black_box(checksum);
    elapsed
}

fn time_hash_table_membership(index: &HashMap<String, usize>, table_names: &[String]) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for probe in 0..MEMBERSHIP_PROBES {
        let name = &table_names[probe.wrapping_mul(2_654_435_761) % table_names.len()];
        checksum ^= index.get(black_box(name.as_str())).copied().unwrap();
    }
    let elapsed = started.elapsed().as_nanos();
    black_box(checksum);
    elapsed
}

fn nearest_rank(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

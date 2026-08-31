use std::hint::black_box;
use std::time::Instant;

use super::super::take_retired_material_reference_fields;

const SAMPLE_PAIRS: usize = 31;
const TABLE_COUNT: usize = 8192;

fn fixture_table(index: usize) -> toml::Table {
    toml::Table::from_iter([
        ("unrelated".to_owned(), toml::Value::Integer(index as i64)),
        (
            "url".to_owned(),
            toml::Value::String(format!("res://legacy/{index}.png")),
        ),
    ])
}

fn legacy_take_fields(values: &mut toml::Table) -> Option<toml::Table> {
    if !values.contains_key("uuid") && !values.contains_key("url") {
        return None;
    }
    let mut exact = toml::Table::new();
    if let Some(uuid) = values.remove("uuid") {
        exact.insert("uuid".to_owned(), uuid);
    }
    if let Some(url) = values.remove("url") {
        exact.insert("url".to_owned(), url);
    }
    Some(exact)
}

fn measure(tables: &[toml::Table], optimized: bool) -> u128 {
    let mut tables = tables.to_vec();
    let started = Instant::now();
    let extracted = tables
        .iter_mut()
        .map(|values| {
            if optimized {
                take_retired_material_reference_fields(black_box(values))
            } else {
                legacy_take_fields(black_box(values))
            }
            .map_or(0, |fields| fields.len())
        })
        .sum::<usize>();
    black_box(extracted);
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
fn optimization_batch_20260829ba_runtime328_single_pass_material_reference_preserves_fields() {
    let mut absent = toml::Table::from_iter([("unrelated".to_owned(), toml::Value::Boolean(true))]);
    assert!(take_retired_material_reference_fields(&mut absent).is_none());
    assert_eq!(absent.len(), 1);

    let mut present = fixture_table(7);
    let reference = take_retired_material_reference_fields(&mut present).expect("legacy fields");
    assert_eq!(reference.keys().collect::<Vec<_>>(), [&"url"]);
    assert_eq!(present.keys().collect::<Vec<_>>(), [&"unrelated"]);
}

#[test]
fn optimization_batch_20260829ba_runtime328_material_slot_skips_contains_probes() {
    let source = include_str!("../document.rs");
    let production = source.split_once("#[cfg(test)]").expect("test boundary").0;
    let helper = production
        .split_once("fn take_retired_material_reference_fields")
        .expect("field helper")
        .1
        .split_once("fn migrate_one_reference")
        .expect("helper boundary")
        .0;

    assert_eq!(helper.matches("values.remove(").count(), 2);
    assert!(!helper.contains("values.contains_key("));
    assert!(helper.contains("(!exact.is_empty()).then_some(exact)"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260829ba_runtime328_single_pass_retired_material_reference_bench() {
    let tables = (0..TABLE_COUNT).map(fixture_table).collect::<Vec<_>>();
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(&tables, false));
            optimized_samples.push(measure(&tables, true));
        } else {
            optimized_samples.push(measure(&tables, true));
            legacy_samples.push(measure(&tables, false));
        }
    }

    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "RUNTIME328_SINGLE_PASS_RETIRED_MATERIAL_REFERENCE_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
tables={TABLE_COUNT} legacy_max_table_probes=4 optimized_max_table_probes=2 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

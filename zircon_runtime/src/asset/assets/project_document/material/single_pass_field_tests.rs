use std::hint::black_box;
use std::time::Instant;

use super::super::take_named_fields;

const FIELD_NAMES: [&str; 5] = ["kind", "guid", "path_hint", "sub", "locator"];
const SAMPLE_PAIRS: usize = 31;
const TABLE_COUNT: usize = 4096;

fn sample_table(index: usize) -> toml::Table {
    toml::Table::from_iter([
        (
            "guid".to_owned(),
            toml::Value::String(format!("guid-{index}")),
        ),
        (
            "path_hint".to_owned(),
            toml::Value::String(format!("assets/{index}.mat")),
        ),
        ("unrelated".to_owned(), toml::Value::Integer(index as i64)),
    ])
}

fn benchmark_table(index: usize) -> toml::Table {
    toml::Table::from_iter([
        (
            "locator".to_owned(),
            toml::Value::String(format!("material://{index}")),
        ),
        ("unrelated".to_owned(), toml::Value::Integer(index as i64)),
    ])
}

fn legacy_take_named_fields(fields: &mut toml::Table, names: &[&str]) -> Option<toml::Table> {
    if !names.iter().any(|name| fields.contains_key(*name)) {
        return None;
    }
    let mut reference = toml::Table::new();
    for name in names {
        if let Some(value) = fields.remove(*name) {
            reference.insert((*name).to_owned(), value);
        }
    }
    Some(reference)
}

fn measure(tables: &[toml::Table], optimized: bool) -> u128 {
    let mut tables = tables.to_vec();
    let started = Instant::now();
    let total = tables
        .iter_mut()
        .map(|fields| {
            if optimized {
                take_named_fields(black_box(fields), &FIELD_NAMES)
            } else {
                legacy_take_named_fields(black_box(fields), &FIELD_NAMES)
            }
            .map_or(0, |value| value.len())
        })
        .sum::<usize>();
    black_box(total);
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
fn optimization_batch_20260829ay_runtime326_material_reference_field_extraction_preserves_absent_and_present_fields(
) {
    let mut absent = toml::Table::from_iter([("unrelated".to_owned(), toml::Value::Boolean(true))]);
    assert!(take_named_fields(&mut absent, &FIELD_NAMES).is_none());
    assert_eq!(absent.len(), 1);

    let mut present = sample_table(7);
    let reference = take_named_fields(&mut present, &FIELD_NAMES).expect("reference fields");
    assert_eq!(reference.len(), 2);
    assert_eq!(present.len(), 1);
    assert!(present.contains_key("unrelated"));
}

#[test]
fn optimization_batch_20260829ay_runtime326_material_reference_field_extraction_scans_the_fixed_name_set_once(
) {
    let mut fields = sample_table(3);
    let reference = take_named_fields(&mut fields, &["guid", "missing", "path_hint"])
        .expect("reference fields");
    assert_eq!(
        reference.keys().collect::<Vec<_>>(),
        [&"guid", &"path_hint"]
    );
    assert_eq!(fields.keys().collect::<Vec<_>>(), [&"unrelated"]);
}

#[test]
#[ignore = "release performance evidence for the managed validation coordinator"]
fn optimization_batch_20260829ay_runtime326_single_pass_material_field_bench() {
    let tables = (0..TABLE_COUNT).map(benchmark_table).collect::<Vec<_>>();
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
        "RUNTIME326_SINGLE_PASS_MATERIAL_FIELD_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
tables={TABLE_COUNT} fixed_names={} legacy_max_table_probes={} optimized_max_table_probes={} \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        FIELD_NAMES.len(),
        FIELD_NAMES.len() * 2,
        FIELD_NAMES.len(),
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

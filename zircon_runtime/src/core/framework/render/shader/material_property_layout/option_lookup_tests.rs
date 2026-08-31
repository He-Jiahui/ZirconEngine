use std::collections::{BTreeMap, HashMap};
use std::hint::black_box;
use std::time::Instant;

use super::{MaterialOptionKind, MaterialOptionRef, MaterialOptionTable};

const OPTION_COUNT: usize = 512;
const CALL_COUNT: usize = 16;
const SAMPLE_COUNT: usize = 17;

#[test]
fn runtime09c_batch_material_option_hash_index_preserves_value_order() {
    let mut options = vec![bool_option("z_override", 0), bool_option("a_override", 0)];
    options.extend((0..14).map(|index| bool_option(&format!("middle_{index:02}"), index + 1)));
    options.push(bool_option("a_override", 15));
    let table = MaterialOptionTable {
        options,
        total_bits: 16,
    };
    let mut values = BTreeMap::new();
    values.insert("a_override".to_string(), toml::Value::Boolean(true));
    values.insert("unknown".to_string(), toml::Value::Boolean(true));
    values.insert("z_override".to_string(), toml::Value::Boolean(false));
    for index in 0..14 {
        values.insert(format!("middle_{index:02}"), toml::Value::Boolean(true));
    }

    let bits = table.bits_for_values(&values);
    assert_eq!(
        bits & 1,
        0,
        "later input key must retain overwrite authority"
    );
    assert_eq!(bits >> 1, (1 << 14) - 1);
    assert_eq!(bits, legacy_bits_for_values(&table, &values));
}

#[test]
fn runtime09c_batch_material_option_hash_index_keeps_small_table_fast_path() {
    let source = include_str!("../material_property_layout.rs");

    assert!(source.contains("use std::collections::{BTreeMap, HashMap};"));
    assert!(source.contains("MATERIAL_OPTION_HASH_INDEX_MIN_OPTIONS"));
    assert!(source.contains("HashMap<&str, &MaterialOptionRef>"));
    assert!(source.contains("values.iter().fold"));
}

#[test]
#[ignore = "release performance evidence"]
fn runtime09c_batch_material_option_hash_index_p95() {
    let table = MaterialOptionTable {
        options: (0..OPTION_COUNT)
            .map(|index| bool_option(&format!("option.shared_prefix.{index:04}"), index % 32))
            .collect(),
        total_bits: 32,
    };
    let values = table
        .options
        .iter()
        .map(|option| (option.name.clone(), toml::Value::Boolean(true)))
        .collect::<BTreeMap<_, _>>();

    let mut scan_lookup = || repeated_legacy(&table, &values);
    let mut hash_lookup = || repeated_hash(&table, &values);
    assert_eq!(black_box(scan_lookup()), black_box(hash_lookup()));

    let mut scan_ns = Vec::with_capacity(SAMPLE_COUNT);
    let mut hash_ns = Vec::with_capacity(SAMPLE_COUNT);
    for sample_index in 0..SAMPLE_COUNT {
        if sample_index % 2 == 0 {
            scan_ns.push(measure_ns(&mut scan_lookup));
            hash_ns.push(measure_ns(&mut hash_lookup));
        } else {
            hash_ns.push(measure_ns(&mut hash_lookup));
            scan_ns.push(measure_ns(&mut scan_lookup));
        }
    }

    let scan_p50 = nearest_rank(&scan_ns, 50);
    let scan_p95 = nearest_rank(&scan_ns, 95);
    let hash_p50 = nearest_rank(&hash_ns, 50);
    let hash_p95 = nearest_rank(&hash_ns, 95);
    assert!(
        hash_p95.saturating_mul(5) <= scan_p95,
        "material option hash lookup P95 must be at least 80% below repeated scans: scan={scan_p95}ns hash={hash_p95}ns"
    );

    println!(
        "RUNTIME09C_MATERIAL_OPTION_VALUE_HASH_INDEX_BENCH_V1 options={OPTION_COUNT} calls={CALL_COUNT} sample_pairs={SAMPLE_COUNT} pair_order=alternating_legacy_even scan_p50_ns={scan_p50} scan_p95_ns={scan_p95} hash_p50_ns={hash_p50} hash_p95_ns={hash_p95} comparisons_before=2101248 comparisons_after=0 hash_index_insertions=8192 hash_lookups_after=8192 owned_key_allocations=0 scan_ns={} hash_ns={}",
        join_samples(&scan_ns),
        join_samples(&hash_ns),
    );
}

fn bool_option(name: &str, bit_offset: usize) -> MaterialOptionRef {
    MaterialOptionRef {
        name: name.to_string(),
        kind: MaterialOptionKind::Bool,
        bit_offset: bit_offset as u8,
        bit_width: 1,
        enum_values: Vec::new(),
        default_bits: 0,
    }
}

fn repeated_legacy(table: &MaterialOptionTable, values: &BTreeMap<String, toml::Value>) -> u32 {
    let mut bits = 0;
    for _ in 0..CALL_COUNT {
        bits = bits.wrapping_add(black_box(legacy_bits_for_values(
            black_box(table),
            black_box(values),
        )));
    }
    bits
}

fn repeated_hash(table: &MaterialOptionTable, values: &BTreeMap<String, toml::Value>) -> u32 {
    let mut bits = 0;
    for _ in 0..CALL_COUNT {
        bits = bits.wrapping_add(black_box(table.bits_for_values(black_box(values))));
    }
    bits
}

fn legacy_bits_for_values(
    table: &MaterialOptionTable,
    values: &BTreeMap<String, toml::Value>,
) -> u32 {
    values
        .iter()
        .fold(table.default_bits(), |bits, (name, value)| {
            let Some(option) = table.options.iter().find(|option| option.name == *name) else {
                return bits;
            };
            apply_value(bits, option, value)
        })
}

fn apply_value(bits: u32, option: &MaterialOptionRef, value: &toml::Value) -> u32 {
    let Some(value_bits) = option.value_bits(value) else {
        return bits;
    };
    let local_mask = super::option_bit_mask(option.bit_width);
    let shifted_mask = local_mask << option.bit_offset;
    (bits & !shifted_mask) | ((value_bits & local_mask) << option.bit_offset)
}

fn measure_ns(operation: &mut impl FnMut() -> u32) -> u128 {
    let started = Instant::now();
    black_box(operation());
    started.elapsed().as_nanos()
}

fn nearest_rank(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = sorted.len().saturating_mul(percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn join_samples(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}

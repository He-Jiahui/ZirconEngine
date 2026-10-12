use std::collections::HashSet;
use std::hint::black_box;
use std::time::Instant;

use zircon_runtime::core::framework::ai::{
    AiBlackboardEntry, AiBlackboardSchemaDescriptor, AiBlackboardValue, AiBlackboardValueType,
    AiManagerError,
};

use super::validate_blackboard_entries;

const BENCHMARK_KEY_COUNT: usize = 256;
const BENCHMARK_ITERATIONS: usize = 16;
const BENCHMARK_PARSE_COUNT: usize = 4_096;
const BENCHMARK_SAMPLE_COUNT: usize = 21;

#[test]
fn blackboard_validation_uses_one_capacity_index_and_preserves_input_error_order() {
    let source = include_str!("../runtime_inputs.rs");
    let validation = function_body(
        source,
        "pub(in crate::manager) fn validate_blackboard_entries(",
        "pub(in crate::manager) fn validate_perception_snapshot(",
    );
    assert!(validation.contains("HashMap::with_capacity(entries.len())"));
    assert!(validation.contains("matching_entry.1 = true;"));
    assert!(!validation.contains("entries.iter().find"));
    assert!(
        !validation.contains("schema\n            .keys\n            .iter()\n            .any")
    );

    let schema = AiBlackboardSchemaDescriptor::new("ordered", "Ordered")
        .with_key("known-a", "integer", true)
        .with_key("known-b", "integer", true);
    let entries = vec![
        AiBlackboardEntry::new("unknown-first", AiBlackboardValue::Integer(1)),
        AiBlackboardEntry::new("known-b", AiBlackboardValue::Integer(2)),
        AiBlackboardEntry::new("known-a", AiBlackboardValue::Integer(3)),
        AiBlackboardEntry::new("unknown-second", AiBlackboardValue::Integer(4)),
    ];

    let error = validate_blackboard_entries(Some(&schema), &entries)
        .expect_err("the first unknown entry must be rejected");
    assert!(matches!(
        error,
        AiManagerError::UnknownBlackboardKey { key, .. } if key == "unknown-first"
    ));
}

#[test]
fn blackboard_value_type_parser_is_borrowed_and_ascii_case_insensitive() {
    let source = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../zircon_runtime/src/core/framework/ai/blackboard.rs"
    ));
    let parser = function_body(source, "pub fn parse(value: &str)", "pub const fn as_str");
    assert!(parser.contains("eq_ignore_ascii_case"));
    assert!(!parser.contains("to_ascii_lowercase"));

    assert_eq!(
        AiBlackboardValueType::parse(" BoOlEaN "),
        Some(AiBlackboardValueType::Bool)
    );
    assert_eq!(
        AiBlackboardValueType::parse("VeCtOr3"),
        Some(AiBlackboardValueType::Vec3)
    );
    assert_eq!(
        AiBlackboardValueType::parse(" ENTITY_ID "),
        Some(AiBlackboardValueType::Entity)
    );
    assert_eq!(AiBlackboardValueType::parse("matrix4"), None);
}

#[test]
fn indexed_blackboard_validation_accepts_reversed_complete_entries() {
    let (schema, entries) = benchmark_fixture(32);
    validate_blackboard_entries(Some(&schema), &entries).expect("complete entries validate");
}

#[test]
#[ignore = "release-only performance evidence"]
fn indexed_blackboard_entry_validation_release_benchmark_evidence() {
    let (schema, entries) = benchmark_fixture(BENCHMARK_KEY_COUNT);
    let (legacy_samples, optimized_samples) = benchmark_paired_samples(
        || {
            for _ in 0..BENCHMARK_ITERATIONS {
                legacy_validate_blackboard_entries(&schema, &entries);
            }
        },
        || {
            for _ in 0..BENCHMARK_ITERATIONS {
                black_box(validate_blackboard_entries(Some(&schema), &entries))
                    .expect("indexed validation succeeds");
            }
        },
    );
    let metrics = metrics(&legacy_samples, &optimized_samples);
    let legacy_key_comparisons = BENCHMARK_KEY_COUNT * (BENCHMARK_KEY_COUNT + 1);
    let optimized_hash_operations = BENCHMARK_KEY_COUNT * 3;
    println!(
        "PERF_RESULT plugins15_indexed_blackboard_entry_validation keys={BENCHMARK_KEY_COUNT} iterations={BENCHMARK_ITERATIONS} sample_pairs={BENCHMARK_SAMPLE_COUNT} sample_order=alternating percentile_method=nearest_rank legacy_key_comparisons_per_iteration={legacy_key_comparisons} optimized_hash_operations_per_iteration={optimized_hash_operations} legacy_index_allocations_per_iteration=1 optimized_index_allocations_per_iteration=1 legacy_p50_ns={} legacy_p95_ns={} optimized_p50_ns={} optimized_p95_ns={} legacy_ns={} optimized_ns={}",
        metrics.legacy_p50,
        metrics.legacy_p95,
        metrics.optimized_p50,
        metrics.optimized_p95,
        metrics.legacy_ns,
        metrics.optimized_ns,
    );
    assert!(
        metrics.optimized_p95.saturating_mul(4) <= metrics.legacy_p95,
        "indexed validation P95 must be at most 25% of nested scan P95"
    );
}

#[test]
#[ignore = "release-only performance evidence"]
fn borrowed_blackboard_value_type_parse_release_benchmark_evidence() {
    const VALUES: [&str; 8] = [
        " Boolean ",
        "INTEGER",
        " f32 ",
        "String",
        " VeCtOr3 ",
        "ENTITY_ID",
        "scalar",
        "unknown",
    ];
    let (legacy_samples, optimized_samples) = benchmark_paired_samples(
        || {
            let mut parsed = 0;
            for index in 0..BENCHMARK_PARSE_COUNT {
                parsed += legacy_parse(VALUES[index % VALUES.len()]).is_some() as usize;
            }
            black_box(parsed)
        },
        || {
            let mut parsed = 0;
            for index in 0..BENCHMARK_PARSE_COUNT {
                parsed +=
                    AiBlackboardValueType::parse(VALUES[index % VALUES.len()]).is_some() as usize;
            }
            black_box(parsed)
        },
    );
    let metrics = metrics(&legacy_samples, &optimized_samples);
    println!(
        "PERF_RESULT plugins15_borrowed_blackboard_value_type_parse parses={BENCHMARK_PARSE_COUNT} sample_pairs={BENCHMARK_SAMPLE_COUNT} sample_order=alternating percentile_method=nearest_rank legacy_lowercase_string_allocations_per_sample={BENCHMARK_PARSE_COUNT} optimized_lowercase_string_allocations_per_sample=0 legacy_p50_ns={} legacy_p95_ns={} optimized_p50_ns={} optimized_p95_ns={} legacy_ns={} optimized_ns={}",
        metrics.legacy_p50,
        metrics.legacy_p95,
        metrics.optimized_p50,
        metrics.optimized_p95,
        metrics.legacy_ns,
        metrics.optimized_ns,
    );
    assert!(
        metrics.optimized_p95.saturating_mul(2) <= metrics.legacy_p95,
        "borrowed value type parse P95 must be at most 50% of allocating parse P95"
    );
}

fn benchmark_fixture(key_count: usize) -> (AiBlackboardSchemaDescriptor, Vec<AiBlackboardEntry>) {
    let mut schema = AiBlackboardSchemaDescriptor::new("benchmark", "Benchmark");
    for index in 0..key_count {
        schema = schema.with_key(format!("key-{index:04}"), "integer", true);
    }
    let entries = (0..key_count)
        .rev()
        .map(|index| {
            AiBlackboardEntry::new(
                format!("key-{index:04}"),
                AiBlackboardValue::Integer(index as i64),
            )
        })
        .collect();
    (schema, entries)
}

fn legacy_validate_blackboard_entries(
    schema: &AiBlackboardSchemaDescriptor,
    entries: &[AiBlackboardEntry],
) {
    let mut seen_entries = HashSet::new();
    for entry in entries {
        assert!(seen_entries.insert(entry.key.as_str()));
    }
    for descriptor in &schema.keys {
        let matching_entry = entries
            .iter()
            .find(|entry| entry.key == descriptor.key)
            .expect("required benchmark entry exists");
        assert_eq!(
            descriptor.expected_value_type(),
            Some(matching_entry.value.value_type())
        );
    }
    for entry in entries {
        assert!(schema
            .keys
            .iter()
            .any(|descriptor| descriptor.key == entry.key));
    }
}

fn legacy_parse(value: &str) -> Option<AiBlackboardValueType> {
    match value.trim().to_ascii_lowercase().as_str() {
        "bool" | "boolean" => Some(AiBlackboardValueType::Bool),
        "integer" | "int" | "i64" => Some(AiBlackboardValueType::Integer),
        "scalar" | "float" | "real" | "f32" => Some(AiBlackboardValueType::Scalar),
        "string" | "str" => Some(AiBlackboardValueType::String),
        "vec3" | "vector3" => Some(AiBlackboardValueType::Vec3),
        "entity" | "entity_id" => Some(AiBlackboardValueType::Entity),
        _ => None,
    }
}

fn function_body<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    let start = source.find(start).expect("function start exists");
    let end = source[start..]
        .find(end)
        .map(|offset| start + offset)
        .expect("function end exists");
    &source[start..end]
}

struct BenchmarkMetrics {
    legacy_p50: u128,
    legacy_p95: u128,
    optimized_p50: u128,
    optimized_p95: u128,
    legacy_ns: String,
    optimized_ns: String,
}

fn metrics(legacy_samples: &[u128], optimized_samples: &[u128]) -> BenchmarkMetrics {
    BenchmarkMetrics {
        legacy_p50: percentile(legacy_samples, 50),
        legacy_p95: percentile(legacy_samples, 95),
        optimized_p50: percentile(optimized_samples, 50),
        optimized_p95: percentile(optimized_samples, 95),
        legacy_ns: samples_csv(legacy_samples),
        optimized_ns: samples_csv(optimized_samples),
    }
}

fn benchmark_paired_samples<L, O>(
    mut legacy: impl FnMut() -> L,
    mut optimized: impl FnMut() -> O,
) -> (Vec<u128>, Vec<u128>) {
    black_box(legacy());
    black_box(optimized());
    let mut legacy_samples = Vec::with_capacity(BENCHMARK_SAMPLE_COUNT);
    let mut optimized_samples = Vec::with_capacity(BENCHMARK_SAMPLE_COUNT);
    for sample_index in 0..BENCHMARK_SAMPLE_COUNT {
        if sample_index % 2 == 0 {
            legacy_samples.push(benchmark_sample(&mut legacy));
            optimized_samples.push(benchmark_sample(&mut optimized));
        } else {
            optimized_samples.push(benchmark_sample(&mut optimized));
            legacy_samples.push(benchmark_sample(&mut legacy));
        }
    }
    (legacy_samples, optimized_samples)
}

fn benchmark_sample<T>(operation: &mut impl FnMut() -> T) -> u128 {
    let started = Instant::now();
    let result = black_box(operation());
    let elapsed = started.elapsed().as_nanos();
    black_box(&result);
    elapsed
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut ordered = samples.to_vec();
    ordered.sort_unstable();
    let index = (ordered.len() * percentile).div_ceil(100) - 1;
    ordered[index]
}

fn samples_csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}

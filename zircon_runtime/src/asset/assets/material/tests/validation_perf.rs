use std::collections::{HashMap, HashSet};
use std::hint::black_box;
use std::time::Instant;

#[test]
fn optimization_batch_ie_runtime616_shader_contract_uses_borrowed_indexes() {
    let source = include_str!("../validation.rs");
    let body = source
        .split("pub fn validate_shader_contract(")
        .nth(1)
        .and_then(|body| body.split("fn is_standard_material_override").next())
        .unwrap();

    assert!(body.contains("SHADER_CONTRACT_INDEX_MIN_COMPARISONS"));
    assert!(body.contains("should_index_contract_lookup"));
    assert!(body.contains("HashMap::with_capacity"));
    assert!(body.contains("HashSet::with_capacity"));
    assert!(body.contains("properties_by_name.get(name.as_str())"));
    assert!(body.contains("texture_binding_names.contains(slot.as_str())"));
    assert!(body.contains("properties.iter().find"));
    assert!(body.contains("texture_bindings.iter().any"));
}

#[test]
fn optimization_batch_ie_runtime616_indexes_only_amortized_contract_lookups() {
    assert!(!super::should_index_contract_lookup(8, 7));
    assert!(super::should_index_contract_lookup(8, 8));
    assert!(!super::should_index_contract_lookup(4_096, 0));
}

#[test]
fn wgsl_capture_admission_uses_parsed_identity_and_span_contracts() {
    let source = include_str!("../validation.rs");
    assert!(source.contains("reflect_declared_shader_resources"));
    assert!(source.contains("texture_binding_matches_kind"));
    assert!(source.contains("find_module_span"));
    assert!(!source.contains("fn captures_name"));
    assert!(!source.contains("source.contains(&slot.name)"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_ie_runtime616_shader_contract_index_p95() {
    const SCHEMA: usize = 4_096;
    const QUERIES: usize = 4_096;
    const SAMPLE_PAIRS: usize = 17;
    let schema = (0..SCHEMA)
        .map(|index| format!("material_property_with_long_name_{index:05}"))
        .collect::<Vec<_>>();
    let queries = (0..QUERIES)
        .rev()
        .map(|index| format!("material_property_with_long_name_{index:05}"))
        .collect::<Vec<_>>();
    let mut retired = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            retired.push(measure_contract_lookup(&schema, &queries, true));
            optimized.push(measure_contract_lookup(&schema, &queries, false));
        } else {
            optimized.push(measure_contract_lookup(&schema, &queries, false));
            retired.push(measure_contract_lookup(&schema, &queries, true));
        }
    }
    let retired_p95_ns = percentile(&retired, 95);
    let optimized_p95_ns = percentile(&optimized, 95);
    println!(
        "RUNTIME616_ADAPTIVE_SHADER_CONTRACT_INDEX_BENCH_V2 sample_pairs={SAMPLE_PAIRS} \
             schema={SCHEMA} queries={QUERIES} retired_p95_ns={retired_p95_ns} \
             optimized_p95_ns={optimized_p95_ns} small_schema=8 small_queries=4 \
             small_index_allocations=0 retired_raw_ns={} optimized_raw_ns={}",
        csv(&retired),
        csv(&optimized)
    );
    assert!(
        optimized_p95_ns.saturating_mul(100) <= retired_p95_ns.saturating_mul(30),
        "borrowed contract index P95 must be at most 30% of nested scans: retired={retired_p95_ns}ns optimized={optimized_p95_ns}ns"
    );
}

fn measure_contract_lookup(schema: &[String], queries: &[String], retired: bool) -> u128 {
    let started = Instant::now();
    let checksum = if retired {
        queries
            .iter()
            .filter(|query| schema.iter().any(|candidate| candidate == *query))
            .count()
    } else {
        let mut names = HashMap::with_capacity(schema.len());
        let mut slots = HashSet::with_capacity(schema.len());
        for (index, name) in schema.iter().enumerate() {
            names.entry(name.as_str()).or_insert(index);
            slots.insert(name.as_str());
        }
        queries
            .iter()
            .filter(|query| names.contains_key(query.as_str()) && slots.contains(query.as_str()))
            .count()
    };
    black_box(checksum);
    started.elapsed().as_nanos().max(1)
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    sorted[(sorted.len() * percentile).div_ceil(100).saturating_sub(1)]
}

fn csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}

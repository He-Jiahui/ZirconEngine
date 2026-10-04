use std::hint::black_box;
use std::time::Instant;

use super::*;

#[test]
fn explicit_layout_usize_values_are_bounded_before_runtime_allocation() {
    let maximum = Value::Integer(MAX_UI_LAYOUT_DISCRETE_VALUE as i64);
    assert_eq!(
        parse_usize(Some(&maximum), "root", "container.columns").unwrap(),
        Some(MAX_UI_LAYOUT_DISCRETE_VALUE)
    );

    let oversized = Value::Integer((MAX_UI_LAYOUT_DISCRETE_VALUE + 1) as i64);
    let error = parse_usize(Some(&oversized), "root", "container.columns").unwrap_err();
    assert!(error.to_string().contains(&format!(
        "container.columns must not exceed {MAX_UI_LAYOUT_DISCRETE_VALUE}"
    )));
}

#[test]
#[ignore = "release-only bounded layout admission benchmark"]
fn bounded_layout_admission_release_benchmark_evidence() {
    const SAMPLE_PAIRS: usize = 21;
    const OPERATIONS_PER_SAMPLE: usize = 64;
    const AUTHORED_TRACK_COUNT: usize = 65_536;

    fn legacy_parse(value: &Value) -> usize {
        value
            .as_integer()
            .and_then(|value| usize::try_from(value).ok())
            .expect("non-negative legacy layout integer")
    }

    fn measure_legacy(value: &Value) -> u128 {
        let started = Instant::now();
        for _ in 0..OPERATIONS_PER_SAMPLE {
            let track_count = legacy_parse(black_box(value));
            black_box(vec![0.0_f32; track_count]);
        }
        started.elapsed().as_nanos().max(1)
    }

    fn measure_optimized(value: &Value) -> u128 {
        let started = Instant::now();
        for _ in 0..OPERATIONS_PER_SAMPLE {
            black_box(parse_usize(
                Some(black_box(value)),
                "benchmark-root",
                "container.columns",
            ));
        }
        started.elapsed().as_nanos().max(1)
    }

    fn percentile(samples: &[u128], percentile: usize) -> u128 {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        let rank = (sorted.len() * percentile).div_ceil(100);
        sorted[rank.saturating_sub(1)]
    }

    fn raw(samples: &[u128]) -> String {
        samples
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }

    let oversized = Value::Integer(AUTHORED_TRACK_COUNT as i64);
    assert!(parse_usize(Some(&oversized), "benchmark-root", "container.columns").is_err());

    for _ in 0..4 {
        black_box(measure_legacy(&oversized));
        black_box(measure_optimized(&oversized));
    }

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure_legacy(&oversized));
            optimized_samples.push(measure_optimized(&oversized));
        } else {
            optimized_samples.push(measure_optimized(&oversized));
            legacy_samples.push(measure_legacy(&oversized));
        }
    }

    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "RUNTIME76_BOUNDED_LAYOUT_ADMISSION_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
operations_per_sample={OPERATIONS_PER_SAMPLE} authored_track_count={AUTHORED_TRACK_COUNT} \
max_discrete_value={MAX_UI_LAYOUT_DISCRETE_VALUE} pair_order=alternating_legacy_even \
legacy_first_pairs=11 optimized_first_pairs=10 \
legacy_track_allocations_per_sample={OPERATIONS_PER_SAMPLE} \
optimized_track_allocations_per_sample=0 legacy_p50_ns={legacy_p50_ns} \
optimized_p50_ns={optimized_p50_ns} legacy_p95_ns={legacy_p95_ns} \
optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
        raw(&legacy_samples),
        raw(&optimized_samples),
    );

    assert!(
        optimized_p95_ns.saturating_mul(4) <= legacy_p95_ns,
        "bounded admission must reduce malicious-track P95 by at least 75%: \
legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );
}

#[test]
fn optimization_batch_em_container_gaps_share_fallback_lookup() {
    let source = include_str!("../parsers.rs");
    let production = source
        .split("#[cfg(test)]")
        .next()
        .expect("layout parser production source");

    assert_eq!(production.matches("parse_axis_gaps(table,").count(), 2);
    let helper = production
        .split("fn parse_axis_gaps(")
        .nth(1)
        .expect("shared container gap parser");
    assert_eq!(helper.matches("table.get(\"gap\")").count(), 1);
}

#[test]
#[ignore = "release-only shared container gap lookup benchmark"]
fn optimization_batch_em_shared_container_gap_lookup_release_benchmark_evidence() {
    const SAMPLE_PAIRS: usize = 17;
    const PARSES_PER_SAMPLE: usize = 32_768;

    fn legacy(table: &toml::map::Map<String, Value>) -> (f32, f32) {
        let horizontal = parse_f32(table.get("horizontal_gap"))
            .or_else(|| parse_f32(table.get("gap")))
            .unwrap_or(0.0);
        let vertical = parse_f32(table.get("vertical_gap"))
            .or_else(|| parse_f32(table.get("gap")))
            .unwrap_or(0.0);
        (horizontal, vertical)
    }

    fn measure_legacy(table: &toml::map::Map<String, Value>) -> u128 {
        let started = Instant::now();
        let mut checksum = 0.0_f32;
        for _ in 0..PARSES_PER_SAMPLE {
            let gaps = black_box(legacy(black_box(table)));
            checksum += gaps.0 + gaps.1;
        }
        black_box(checksum);
        started.elapsed().as_nanos().max(1)
    }

    fn measure_optimized(table: &toml::map::Map<String, Value>) -> u128 {
        let started = Instant::now();
        let mut checksum = 0.0_f32;
        for _ in 0..PARSES_PER_SAMPLE {
            let gaps = black_box(parse_axis_gaps(
                black_box(table),
                "horizontal_gap",
                "vertical_gap",
            ));
            checksum += gaps.0 + gaps.1;
        }
        black_box(checksum);
        started.elapsed().as_nanos().max(1)
    }

    fn percentile(samples: &[u128], percentile: usize) -> u128 {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        let rank = (sorted.len() * percentile).div_ceil(100);
        sorted[rank.saturating_sub(1)]
    }

    fn raw(samples: &[u128]) -> String {
        samples
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }

    let mut table = toml::map::Map::new();
    for index in 0..512 {
        table.insert(format!("layout_filler_{index:04}"), Value::Integer(index));
    }
    table.insert("gap".to_string(), Value::Float(7.5));
    assert_eq!(
        legacy(&table),
        parse_axis_gaps(&table, "horizontal_gap", "vertical_gap")
    );
    for _ in 0..4 {
        black_box(measure_legacy(&table));
        black_box(measure_optimized(&table));
    }

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for sample in 0..SAMPLE_PAIRS {
        if sample % 2 == 0 {
            legacy_samples.push(measure_legacy(&table));
            optimized_samples.push(measure_optimized(&table));
        } else {
            optimized_samples.push(measure_optimized(&table));
            legacy_samples.push(measure_legacy(&table));
        }
    }

    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "RUNTIME447_SHARED_CONTAINER_GAP_LOOKUP_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
             parses_per_sample={PARSES_PER_SAMPLE} table_entries={} \
             pair_order=alternating_legacy_even legacy_shared_gap_lookups_per_parse=2 \
             optimized_shared_gap_lookups_per_parse=1 legacy_p50_ns={legacy_p50_ns} \
             optimized_p50_ns={optimized_p50_ns} legacy_p95_ns={legacy_p95_ns} \
             optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
        table.len(),
        raw(&legacy_samples),
        raw(&optimized_samples),
    );

    assert!(
            optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(85),
            "shared container gap lookup must reduce P95 by at least 15%: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
        );
}

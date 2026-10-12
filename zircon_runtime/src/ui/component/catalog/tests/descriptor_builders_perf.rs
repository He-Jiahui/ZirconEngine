use std::hint::black_box;
use std::time::Instant;

use toml::Value;

use super::table_value;

#[test]
fn optimization_batch_20260831gs_runtime574_table_value_preserves_entries() {
    let value = table_value([
        ("kind", Value::String("ScrollableBox".to_owned())),
        ("axis", Value::String("Vertical".to_owned())),
        ("gap", Value::Integer(0)),
        ("scrollbar_visibility", Value::String("Auto".to_owned())),
    ]);
    let table = value.as_table().expect("table value");
    assert_eq!(table["kind"].as_str(), Some("ScrollableBox"));
    assert_eq!(table["axis"].as_str(), Some("Vertical"));
    assert_eq!(table["gap"].as_integer(), Some(0));
    assert_eq!(table["scrollbar_visibility"].as_str(), Some("Auto"));
}

#[test]
#[ignore = "release performance evidence"]
fn optimization_batch_20260831gs_runtime574_table_value_move_benchmark() {
    const SAMPLE_PAIRS: usize = 21;
    const ITERATIONS: usize = 120_000;
    let mut legacy_ns = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_ns = Vec::with_capacity(SAMPLE_PAIRS);
    let mut checksum = 0usize;
    for sample_index in 0..SAMPLE_PAIRS {
        if sample_index % 2 == 0 {
            let (elapsed, value) = measure(ITERATIONS, legacy_table_len);
            legacy_ns.push(elapsed);
            checksum ^= value;
            let (elapsed, value) = measure(ITERATIONS, optimized_table_len);
            optimized_ns.push(elapsed);
            checksum ^= value;
        } else {
            let (elapsed, value) = measure(ITERATIONS, optimized_table_len);
            optimized_ns.push(elapsed);
            checksum ^= value;
            let (elapsed, value) = measure(ITERATIONS, legacy_table_len);
            legacy_ns.push(elapsed);
            checksum ^= value;
        }
    }
    let legacy_p95_ns = nearest_rank(&legacy_ns, 95);
    let optimized_p95_ns = nearest_rank(&optimized_ns, 95);
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(85),
        "moved table values P95 must be at least 15% below cloned values: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );
    println!(
        "RUNTIME574_TABLE_VALUE_MOVE_BENCH_V1 sample_pairs={SAMPLE_PAIRS} iterations={ITERATIONS} checksum={checksum} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} legacy_ns={} optimized_ns={}",
        join_samples(&legacy_ns),
        join_samples(&optimized_ns),
    );

    fn measure(iterations: usize, operation: fn() -> usize) -> (u128, usize) {
        let started = Instant::now();
        let mut checksum = 0usize;
        for _ in 0..iterations {
            checksum = checksum.wrapping_add(operation());
        }
        (started.elapsed().as_nanos(), black_box(checksum))
    }

    fn entries() -> [(&'static str, Value); 4] {
        [
            ("kind", Value::String("ScrollableBox".to_owned())),
            ("axis", Value::String("Vertical".to_owned())),
            ("gap", Value::Integer(0)),
            ("scrollbar_visibility", Value::String("Auto".to_owned())),
        ]
    }

    fn legacy_table_len() -> usize {
        let entries = entries();
        let value = Value::Table(
            entries
                .iter()
                .map(|(key, value)| ((*key).to_owned(), value.clone()))
                .collect(),
        );
        black_box(value.as_table().map_or(0, |table| table.len()))
    }

    fn optimized_table_len() -> usize {
        let value = table_value(entries());
        black_box(value.as_table().map_or(0, |table| table.len()))
    }

    fn nearest_rank(samples: &[u128], percentile: usize) -> u128 {
        let mut ordered = samples.to_vec();
        ordered.sort_unstable();
        let rank = (ordered.len() * percentile).div_ceil(100).max(1);
        ordered[rank - 1]
    }

    fn join_samples(samples: &[u128]) -> String {
        samples
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }
}

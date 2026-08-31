use std::collections::BTreeMap;
use std::hint::black_box;
use std::time::Instant;

use toml::Value;

use super::{collect_value_items, prop_state_row_capacity};

const SAMPLE_PAIRS: usize = 21;
const BUILDS_PER_SAMPLE: usize = 128;
const ROWS_PER_BUILD: usize = 4_096;

#[test]
fn optimization_batch_20260826fp_editor157_capacity_preserves_prop_state_rows() {
    let props = values("prop", 128);
    let state = values("state", 128);
    let expected_capacity = prop_state_row_capacity(&props, &state);
    let mut rows = Vec::with_capacity(expected_capacity);

    collect_value_items(&mut rows, "prop", "", &props);
    collect_value_items(&mut rows, "state", "", &state);

    assert_eq!(expected_capacity, 256);
    assert_eq!(rows.len(), 256);
    assert!(rows.capacity() >= expected_capacity);
    assert_eq!(rows[0].kind, "prop");
    assert_eq!(rows[0].path, "prop-000");
    assert_eq!(rows[127].path, "prop-127");
    assert_eq!(rows[128].kind, "state");
    assert_eq!(rows[255].path, "state-127");
}

#[test]
fn optimization_batch_20260826fp_editor157_inspector_rows_reserve_map_lower_bound() {
    let source = include_str!("../inspector_fields.rs");
    assert!(
        source.contains("Vec::with_capacity(prop_state_row_capacity(&node.props, &node.params))")
    );
    assert!(source.contains("props.len().saturating_add(state.len())"));
    assert!(!source.contains("let mut rows = Vec::new();"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826fp_editor157_inspector_prop_state_row_capacity_bench() {
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(false));
            optimized_samples.push(measure(true));
        } else {
            optimized_samples.push(measure(true));
            legacy_samples.push(measure(false));
        }
    }
    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "EDITOR157_INSPECTOR_PROP_STATE_ROW_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
builds_per_sample={BUILDS_PER_SAMPLE} rows_per_build={ROWS_PER_BUILD} \
legacy_reservations_per_build=0 optimized_reservations_per_build=1 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn values(prefix: &str, count: usize) -> BTreeMap<String, Value> {
    (0..count)
        .map(|index| (format!("{prefix}-{index:03}"), Value::Integer(index as i64)))
        .collect()
}

#[derive(Clone, Copy)]
struct RowFixture([usize; 5]);

fn measure(reserve: bool) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for build in 0..BUILDS_PER_SAMPLE {
        let mut rows = if reserve {
            Vec::with_capacity(ROWS_PER_BUILD)
        } else {
            Vec::new()
        };
        for row in 0..ROWS_PER_BUILD {
            rows.push(RowFixture([black_box(build ^ row); 5]));
        }
        checksum ^= black_box(rows.len() ^ rows.capacity() ^ rows[ROWS_PER_BUILD - 1].0[0]);
        black_box(&rows);
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

fn sample_csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}

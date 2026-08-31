use serde_json::Value;

/// Recursively orders every object key before the text formatter runs.
pub(in crate::serialization) fn canonicalize_value(mut value: Value) -> Value {
    canonicalize_value_in_place(&mut value);
    value
}

fn canonicalize_value_in_place(value: &mut Value) {
    match value {
        Value::Array(values) => {
            for value in values {
                canonicalize_nested_value(value);
            }
        }
        Value::Object(values) => {
            for value in values.values_mut() {
                canonicalize_nested_value(value);
            }
            values.sort_keys();
        }
        _ => {}
    }
}

#[inline]
fn canonicalize_nested_value(value: &mut Value) {
    if matches!(value, Value::Array(_) | Value::Object(_)) {
        canonicalize_value_in_place(value);
    }
}

#[cfg(test)]
mod performance_tests {
    use std::hint::black_box;
    use std::time::Instant;

    use serde_json::{Map, Value};

    use super::canonicalize_value;

    const PERF_ENTRY_COUNT: usize = 1_024;
    const PERF_SAMPLE_PAIRS: usize = 21;

    #[test]
    #[ignore = "release performance evidence"]
    fn canonical_objects_reuse_owned_maps() {
        let fixture = object_fixture();
        assert_eq!(
            legacy_canonicalize_value(fixture.clone()),
            canonicalize_value(fixture.clone())
        );
        let mut legacy_ns = Vec::with_capacity(PERF_SAMPLE_PAIRS);
        let mut optimized_ns = Vec::with_capacity(PERF_SAMPLE_PAIRS);

        for sample in 0..PERF_SAMPLE_PAIRS {
            let legacy_input = fixture.clone();
            let optimized_input = fixture.clone();
            let (legacy, optimized) = if sample % 2 == 0 {
                (
                    measure_once(|| legacy_canonicalize_value(legacy_input)),
                    measure_once(|| canonicalize_value(optimized_input)),
                )
            } else {
                let optimized = measure_once(|| canonicalize_value(optimized_input));
                let legacy = measure_once(|| legacy_canonicalize_value(legacy_input));
                (legacy, optimized)
            };
            legacy_ns.push(legacy);
            optimized_ns.push(optimized);
        }

        let legacy_p50 = percentile(&legacy_ns, 50);
        let legacy_p95 = percentile(&legacy_ns, 95);
        let optimized_p50 = percentile(&optimized_ns, 50);
        let optimized_p95 = percentile(&optimized_ns, 95);
        println!(
            "PERF_RESULT runtime_interface02_canonical_object_reuse legacy_p50_ns={legacy_p50} legacy_p95_ns={legacy_p95} optimized_p50_ns={optimized_p50} optimized_p95_ns={optimized_p95} object_entries={PERF_ENTRY_COUNT} samples={PERF_SAMPLE_PAIRS} legacy_transient_collections_per_object=2 optimized_transient_collections_per_object=0 legacy_value_slot_rewrites_per_child=2 optimized_value_slot_rewrites_per_child=0"
        );
        assert!(
            optimized_p95.saturating_mul(100) <= legacy_p95.saturating_mul(60),
            "optimized P95 {optimized_p95}ns must be at most 60% of legacy P95 {legacy_p95}ns"
        );
    }

    fn object_fixture() -> Value {
        let mut values = Map::new();
        for index in (0..PERF_ENTRY_COUNT).rev() {
            values.insert(
                format!("field-{index:04}"),
                Value::Array(vec![Value::from(index as u64), Value::Bool(index % 2 == 0)]),
            );
        }
        Value::Object(values)
    }

    fn legacy_canonicalize_value(value: Value) -> Value {
        match value {
            Value::Array(values) => {
                Value::Array(values.into_iter().map(legacy_canonicalize_value).collect())
            }
            Value::Object(values) => {
                let mut entries = values.into_iter().collect::<Vec<_>>();
                entries.sort_unstable_by(|left, right| left.0.cmp(&right.0));
                Value::Object(Map::from_iter(
                    entries
                        .into_iter()
                        .map(|(key, value)| (key, legacy_canonicalize_value(value))),
                ))
            }
            value => value,
        }
    }

    fn measure_once<T>(canonicalize: impl FnOnce() -> T) -> u64 {
        let started = Instant::now();
        black_box(canonicalize());
        started.elapsed().as_nanos() as u64
    }

    fn percentile(samples: &[u64], percentile: usize) -> u64 {
        let mut ordered = samples.to_vec();
        ordered.sort_unstable();
        let rank = ordered
            .len()
            .saturating_mul(percentile)
            .div_ceil(100)
            .saturating_sub(1);
        ordered[rank]
    }
}

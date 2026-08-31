use std::hash::{Hash, Hasher};
use std::hint::black_box;
use std::time::Instant;

use super::*;

const KEY_PAYLOAD_BYTES: usize = 32 * 1024;
const OPERATIONS_PER_SAMPLE: usize = 512;
const SAMPLE_PAIRS: usize = 21;

#[test]
fn optimization_batch_20260826hi_runtime255_preserves_hash_value_updates() {
    let existing = "camera-main".to_string();
    let inserted = "camera-secondary".to_string();
    let mut values = HashMap::from([(existing.clone(), 7usize)]);

    set_borrowed_hash_value(&mut values, &existing, 11);
    set_borrowed_hash_value(&mut values, &inserted, 13);

    assert_eq!(values.get(&existing), Some(&11));
    assert_eq!(values.get(&inserted), Some(&13));
    assert_eq!(values.len(), 2);
}

#[test]
fn optimization_batch_20260826hi_runtime255_reuses_existing_product_report_keys() {
    let source = include_str!("../product_reports.rs");
    let start = source
        .find("fn set_borrowed_hash_value")
        .expect("set_borrowed_hash_value function");
    let end = source[start..]
        .find("\n#[cfg(test)]")
        .map(|offset| start + offset)
        .expect("test module boundary");
    let helper = &source[start..end];

    assert!(helper.contains("values.get_mut(key)"));
    assert!(helper.contains("values.insert(key.clone(), value)"));
    assert_eq!(source.matches("set_borrowed_hash_value(").count(), 3);
}

#[test]
#[ignore = "managed release performance evidence"]
fn optimization_batch_20260826hi_runtime255_product_report_key_reuse_release_benchmark() {
    let key = ExpensiveKey {
        identity: 17,
        payload: "camera-history".repeat(KEY_PAYLOAD_BYTES / "camera-history".len()),
    };
    let baseline = HashMap::from([(key.clone(), 0usize)]);
    let mut legacy = baseline.clone();
    let mut optimized = baseline;

    let mut legacy_ns = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_ns = Vec::with_capacity(SAMPLE_PAIRS);
    for sample_index in 0..SAMPLE_PAIRS {
        let mut measure_legacy = || {
            let started = Instant::now();
            for value in 0..OPERATIONS_PER_SAMPLE {
                legacy_set_hash_value(black_box(&mut legacy), black_box(&key), value);
            }
            legacy_ns.push(started.elapsed().as_nanos().max(1));
        };
        let mut measure_optimized = || {
            let started = Instant::now();
            for value in 0..OPERATIONS_PER_SAMPLE {
                set_borrowed_hash_value(black_box(&mut optimized), black_box(&key), value);
            }
            optimized_ns.push(started.elapsed().as_nanos().max(1));
        };
        if sample_index % 2 == 0 {
            measure_legacy();
            measure_optimized();
        } else {
            measure_optimized();
            measure_legacy();
        }
    }
    assert_eq!(legacy, optimized);

    let legacy_p50_ns = percentile(&legacy_ns, 50);
    let legacy_p95_ns = percentile(&legacy_ns, 95);
    let optimized_p50_ns = percentile(&optimized_ns, 50);
    let optimized_p95_ns = percentile(&optimized_ns, 95);
    println!(
        "RUNTIME255_PRODUCT_REPORT_KEY_REUSE_BENCH_V1 \
         key_payload_bytes={KEY_PAYLOAD_BYTES} operations_per_sample={OPERATIONS_PER_SAMPLE} \
         sample_pairs={SAMPLE_PAIRS} legacy_p50_ns={legacy_p50_ns} \
         legacy_p95_ns={legacy_p95_ns} optimized_p50_ns={optimized_p50_ns} \
         optimized_p95_ns={optimized_p95_ns} legacy_ns={} optimized_ns={}",
        samples(&legacy_ns),
        samples(&optimized_ns),
    );
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70),
        "optimized P95 {optimized_p95_ns}ns must be at most 70% of legacy P95 {legacy_p95_ns}ns"
    );
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ExpensiveKey {
    identity: u64,
    payload: String,
}

impl Hash for ExpensiveKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.identity.hash(state);
    }
}

fn legacy_set_hash_value<K, V>(values: &mut HashMap<K, V>, key: &K, value: V)
where
    K: Clone + Eq + Hash,
{
    values.insert(key.clone(), value);
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut ordered = samples.to_vec();
    ordered.sort_unstable();
    let rank = ordered.len().saturating_mul(percentile).div_ceil(100);
    ordered[rank.saturating_sub(1)]
}

fn samples(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}

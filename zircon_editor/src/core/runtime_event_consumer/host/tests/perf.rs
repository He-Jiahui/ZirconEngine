use std::collections::{BTreeSet, HashSet};
use std::hint::black_box;
use std::time::Instant;

const ACTIVE_COUNT: usize = 32_768;
const DESIRED_COUNT: usize = 65_536;
const SAMPLE_PAIRS: usize = 17;

#[test]
fn optimization_batch_iu_editor631_preallocates_reconcile_membership_and_rollback() {
    let source = include_str!("../../host.rs");
    let reconcile_body = source
        .split("fn reconcile_enabled_capabilities_inner")
        .nth(1)
        .expect("runtime consumer reconcile remains present")
        .split("pub fn last_pump_report")
        .next()
        .expect("runtime consumer reconcile remains bounded");

    assert!(reconcile_body.contains("HashSet::with_capacity(active.len())"));
    assert!(reconcile_body.contains("let mut added = Vec::with_capacity(desired.len());"));
    assert!(!reconcile_body.contains(".collect::<BTreeSet<_>>()"));
    assert!(!reconcile_body.contains("let mut added = Vec::new();"));
}

#[test]
#[ignore = "release performance gate"]
fn optimization_batch_iu_editor631_hash_reconcile_membership_benchmark() {
    let active = (0..ACTIVE_COUNT)
        .map(|index| format!("active.consumer.{index:08}"))
        .collect::<Vec<_>>();
    let desired = (0..DESIRED_COUNT)
        .map(|index| format!("desired.consumer.{index:08}"))
        .collect::<Vec<_>>();
    for _ in 0..4 {
        black_box(measure_reconcile_membership(&active, &desired, false));
        black_box(measure_reconcile_membership(&active, &desired, true));
    }

    let mut ordered_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut hash_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            ordered_samples.push(measure_reconcile_membership(&active, &desired, false));
            hash_samples.push(measure_reconcile_membership(&active, &desired, true));
        } else {
            hash_samples.push(measure_reconcile_membership(&active, &desired, true));
            ordered_samples.push(measure_reconcile_membership(&active, &desired, false));
        }
    }

    let ordered_p95 = percentile(&ordered_samples, 95);
    let hash_p95 = percentile(&hash_samples, 95);
    let improvement_percent =
        ordered_p95.saturating_sub(hash_p95).saturating_mul(100) / ordered_p95.max(1);
    println!(
        "EDITOR631_HASH_RUNTIME_CONSUMER_RECONCILE_BENCH_V1 sample_pairs={SAMPLE_PAIRS} active_count={ACTIVE_COUNT} desired_count={DESIRED_COUNT} ordered_ns={} hash_ns={} ordered_p95_ns={ordered_p95} hash_p95_ns={hash_p95} improvement_percent={improvement_percent} threshold_percent=60",
        csv(&ordered_samples),
        csv(&hash_samples),
    );
    assert!(hash_p95 <= ordered_p95 * 40 / 100);
}

fn measure_reconcile_membership(active: &[String], desired: &[String], hash: bool) -> u128 {
    let started = Instant::now();
    if hash {
        let mut existing = HashSet::with_capacity(active.len());
        existing.extend(active.iter().map(String::as_str));
        let mut added = Vec::with_capacity(desired.len());
        for consumer_id in desired {
            if !existing.contains(black_box(consumer_id.as_str())) {
                added.push(consumer_id.as_str());
            }
        }
        black_box((existing, added));
    } else {
        let existing = active.iter().map(String::as_str).collect::<BTreeSet<_>>();
        let mut added = Vec::new();
        for consumer_id in desired {
            if !existing.contains(black_box(consumer_id.as_str())) {
                added.push(consumer_id.as_str());
            }
        }
        black_box((existing, added));
    }
    started.elapsed().as_nanos().max(1)
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}

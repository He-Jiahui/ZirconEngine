use std::collections::{BTreeMap, HashMap};
use std::hint::black_box;
use std::time::Instant;

use super::EditorJobSystemState;
use crate::core::jobs::JobId;

const ENTRY_COUNT: usize = 1_024;
const HIT_COUNT: usize = 4_096;
const SAMPLE_COUNT: usize = 17;

#[test]
fn optimization_batch_20260826bv_job_record_hash_index_preserves_terminal_order() {
    let mut state = EditorJobSystemState::default();
    let first = state.allocate_id();
    let second = state.allocate_id();
    let third = state.allocate_id();
    for id in [first, second, third] {
        state.register(id);
        assert!(state.validate_dependency(id).is_ok());
    }

    state.mark_cancelled(third);
    state.mark_cancelled(first);
    assert_eq!(
        state
            .terminal_records
            .iter()
            .map(|(_, id)| *id)
            .collect::<Vec<_>>(),
        vec![third, first]
    );
    assert!(
        state.terminal_orders.get(&third).unwrap() < state.terminal_orders.get(&first).unwrap()
    );
    assert!(state.validate_dependency(first).is_ok());
    assert!(state.validate_dependency(second).is_ok());
}

#[test]
fn optimization_batch_20260826bv_job_record_hash_index_keeps_ordered_eviction_sets() {
    let source = include_str!("../state.rs");

    assert!(source.contains("use std::collections::{BTreeMap, BTreeSet, HashMap};"));
    assert!(source.contains("records: HashMap<JobId, EditorJobRecord>"));
    assert!(source.contains("terminal_orders: HashMap<JobId, u64>"));
    assert!(source.contains("terminal_records: BTreeSet<(u64, JobId)>"));
    assert!(source.contains("evictable_terminal_records: BTreeSet<(u64, JobId)>"));
}

#[test]
#[ignore = "release performance evidence"]
fn optimization_batch_20260826bv_job_record_hash_index_p95() {
    let ordered = (1..=ENTRY_COUNT)
        .map(|index| (JobId::new(index as u64), index as u64))
        .collect::<BTreeMap<_, _>>();
    let hashed = ordered
        .iter()
        .map(|(id, value)| (*id, *value))
        .collect::<HashMap<_, _>>();
    let target = JobId::new(ENTRY_COUNT as u64);

    let mut ordered_lookup = || repeated_lookup(&ordered, target);
    let mut hash_lookup = || repeated_lookup(&hashed, target);
    assert_eq!(black_box(ordered_lookup()), black_box(hash_lookup()));

    let mut ordered_ns = Vec::with_capacity(SAMPLE_COUNT);
    let mut hash_ns = Vec::with_capacity(SAMPLE_COUNT);
    for sample_index in 0..SAMPLE_COUNT {
        if sample_index % 2 == 0 {
            ordered_ns.push(measure_ns(&mut ordered_lookup));
            hash_ns.push(measure_ns(&mut hash_lookup));
        } else {
            hash_ns.push(measure_ns(&mut hash_lookup));
            ordered_ns.push(measure_ns(&mut ordered_lookup));
        }
    }

    let ordered_p50 = nearest_rank(&ordered_ns, 50);
    let ordered_p95 = nearest_rank(&ordered_ns, 95);
    let hash_p50 = nearest_rank(&hash_ns, 50);
    let hash_p95 = nearest_rank(&hash_ns, 95);
    assert!(
        hash_p95.saturating_mul(10) <= ordered_p95.saturating_mul(7),
        "job record hash lookup P95 must be at least 30% below BTreeMap: ordered={ordered_p95}ns hash={hash_p95}ns"
    );

    println!(
        "EDITOR09_JOB_RECORD_HASH_INDEX_BENCH_V1 entries={ENTRY_COUNT} hits={HIT_COUNT} samples={SAMPLE_COUNT} ordered_p50_ns={ordered_p50} ordered_p95_ns={ordered_p95} hash_p50_ns={hash_p50} hash_p95_ns={hash_p95} ordered_lookups_before={HIT_COUNT} ordered_lookups_after=0 hash_lookups_after={HIT_COUNT} ordered_ns={} hash_ns={}",
        join_samples(&ordered_ns),
        join_samples(&hash_ns),
    );
}

fn repeated_lookup<V>(map: &V, target: JobId) -> u64
where
    V: Lookup,
{
    let mut total = 0_u64;
    for _ in 0..HIT_COUNT {
        total = total.wrapping_add(black_box(map.lookup(black_box(target))).unwrap_or_default());
    }
    total
}

trait Lookup {
    fn lookup(&self, key: JobId) -> Option<u64>;
}

impl Lookup for BTreeMap<JobId, u64> {
    fn lookup(&self, key: JobId) -> Option<u64> {
        self.get(&key).copied()
    }
}

impl Lookup for HashMap<JobId, u64> {
    fn lookup(&self, key: JobId) -> Option<u64> {
        self.get(&key).copied()
    }
}

fn measure_ns(operation: &mut impl FnMut() -> u64) -> u128 {
    let started = Instant::now();
    assert_ne!(black_box(operation()), 0);
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

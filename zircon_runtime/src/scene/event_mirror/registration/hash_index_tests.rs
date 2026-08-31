use std::collections::{BTreeMap, HashMap};
use std::hint::black_box;
use std::time::Instant;

use serde::Serialize;

use super::{RuntimeEventMirrorRegistration, RuntimeEventMirrorRegistry};

const ENTRY_COUNT: usize = 4_096;
const HIT_COUNT: usize = 4_096;
const SAMPLE_COUNT: usize = 17;

#[derive(Serialize)]
struct FirstEvent;

#[derive(Serialize)]
struct SecondEvent;

#[test]
fn runtime_hash_recovery_batch_runtime54_reader_count_hash_index_isolates_events() {
    let mut registry = RuntimeEventMirrorRegistry::default();
    registry
        .insert(RuntimeEventMirrorRegistration::typed::<FirstEvent>(
            "editor.events.first",
            "zircon.editor.first.v1",
        ))
        .unwrap();
    registry
        .insert(RuntimeEventMirrorRegistration::typed::<SecondEvent>(
            "editor.events.second",
            "zircon.editor.second.v1",
        ))
        .unwrap();

    assert_eq!(registry.increment_reader("editor.events.first"), Ok(1));
    assert_eq!(registry.increment_reader("editor.events.second"), Ok(1));
    assert_eq!(registry.increment_reader("editor.events.second"), Ok(2));
    assert_eq!(
        registry
            .lifecycle_diagnostics("editor.events.first")
            .unwrap()
            .reader_count,
        1
    );
    assert_eq!(
        registry
            .lifecycle_diagnostics("editor.events.second")
            .unwrap()
            .reader_count,
        2
    );
    assert!(format!("{registry:?}")
        .contains("reader_counts: {\"editor.events.first\": 1, \"editor.events.second\": 2}"));
    assert_eq!(registry.decrement_reader("editor.events.second"), Ok(1));
}

#[test]
fn runtime_hash_recovery_batch_runtime54_reader_count_hash_index_preserves_registration_order_owner(
) {
    let source = include_str!("../registration.rs");

    assert!(source.contains("use std::collections::{BTreeMap, HashMap};"));
    assert!(source.contains("registrations: BTreeMap<String, RuntimeEventMirrorRegistration>"));
    assert!(source.contains("reader_counts: HashMap<String, u32>"));
    assert!(!source.contains("reader_counts.keys()"));
    assert!(!source.contains("reader_counts.values()"));
    assert!(!source.contains("reader_counts.iter()"));
}

#[test]
#[ignore = "release performance evidence"]
fn runtime_hash_recovery_batch_runtime54_reader_count_hash_index_p95() {
    let event_ids = (0..ENTRY_COUNT)
        .map(|index| format!("editor.events.shared.reader.count.prefix.{index:04}"))
        .collect::<Vec<_>>();
    let ordered = event_ids
        .iter()
        .cloned()
        .enumerate()
        .map(|(index, event_id)| (event_id, index + 1))
        .collect::<BTreeMap<_, _>>();
    let hashed = ordered
        .iter()
        .map(|(event_id, value)| (event_id.clone(), *value))
        .collect::<HashMap<_, _>>();
    let target = event_ids.last().unwrap().as_str();

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
        "event mirror reader-count hash lookup P95 must be at least 30% below BTreeMap: ordered={ordered_p95}ns hash={hash_p95}ns"
    );

    println!(
        "RUNTIME54_EVENT_MIRROR_READER_COUNT_HASH_INDEX_BENCH_V1 entries={ENTRY_COUNT} hits={HIT_COUNT} samples={SAMPLE_COUNT} ordered_p50_ns={ordered_p50} ordered_p95_ns={ordered_p95} hash_p50_ns={hash_p50} hash_p95_ns={hash_p95} ordered_lookups_before={HIT_COUNT} ordered_lookups_after=0 hash_lookups_after={HIT_COUNT} ordered_ns={} hash_ns={}",
        join_samples(&ordered_ns),
        join_samples(&hash_ns),
    );
}

fn repeated_lookup<V>(map: &V, target: &str) -> usize
where
    V: Lookup,
{
    let mut total = 0_usize;
    for _ in 0..HIT_COUNT {
        total = total.wrapping_add(black_box(map.lookup(black_box(target))).unwrap_or_default());
    }
    total
}

trait Lookup {
    fn lookup(&self, key: &str) -> Option<usize>;
}

impl Lookup for BTreeMap<String, usize> {
    fn lookup(&self, key: &str) -> Option<usize> {
        self.get(key).copied()
    }
}

impl Lookup for HashMap<String, usize> {
    fn lookup(&self, key: &str) -> Option<usize> {
        self.get(key).copied()
    }
}

fn measure_ns(operation: &mut impl FnMut() -> usize) -> u128 {
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

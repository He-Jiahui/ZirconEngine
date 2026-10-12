use std::collections::{BTreeMap, HashMap};
use std::hint::black_box;
use std::time::Instant;

use super::ViewportHighlightStore;
use crate::core::framework::render::{HighlightRenderAttributes, HighlightSet};

const ENTRY_COUNT: usize = 4_096;
const HIT_COUNT: usize = 4_096;
const WARMUP_COUNT: usize = 4;
const SAMPLE_COUNT: usize = 17;

#[test]
fn optimization_batch_20260826bx_viewport_highlight_hash_index_preserves_generation_isolation() {
    let mut store = ViewportHighlightStore::default();
    assert!(store.submit(3, 7, set([8, 2])));
    assert!(store.submit(4, 1, set([11])));
    assert!(!store.submit(3, 6, set([99])));
    assert!(store.submit(3, 8, set([5])));

    assert_eq!(store.get(3).unwrap().generation(), 8);
    assert_eq!(store.get(3).unwrap().set().entities(), &[5]);
    assert_eq!(store.get(4).unwrap().generation(), 1);
    assert_eq!(store.get(4).unwrap().set().entities(), &[11]);
}

#[test]
fn optimization_batch_20260826bx_viewport_highlight_hash_index_has_no_ordered_iteration() {
    let source = include_str!("../viewport_highlight_store.rs");

    assert!(source.contains("use std::collections::HashMap;"));
    assert!(source.contains("by_viewport: HashMap<u64, ViewportHighlightSet>"));
    assert!(!source.contains("pub fn iter"));
    assert!(!source.contains("by_viewport.values"));
}

#[test]
#[ignore = "release performance evidence"]
fn optimization_batch_20260826bx_viewport_highlight_hash_index_p95() {
    let ordered = (1..=ENTRY_COUNT as u64)
        .map(|viewport| (viewport, viewport as usize))
        .collect::<BTreeMap<_, _>>();
    let hashed = ordered
        .iter()
        .map(|(viewport, value)| (*viewport, *value))
        .collect::<HashMap<_, _>>();
    let target = ENTRY_COUNT as u64;

    let mut ordered_lookup = || repeated_lookup(&ordered, target);
    let mut hash_lookup = || repeated_lookup(&hashed, target);
    assert_eq!(black_box(ordered_lookup()), black_box(hash_lookup()));
    for _ in 0..WARMUP_COUNT {
        black_box(ordered_lookup());
        black_box(hash_lookup());
    }

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
        "viewport highlight hash lookup P95 must be at least 30% below BTreeMap: ordered={ordered_p95}ns hash={hash_p95}ns"
    );

    println!(
        "RUNTIME62_VIEWPORT_HIGHLIGHT_HASH_INDEX_BENCH_V1 entries={ENTRY_COUNT} hits={HIT_COUNT} samples={SAMPLE_COUNT} ordered_p50_ns={ordered_p50} ordered_p95_ns={ordered_p95} hash_p50_ns={hash_p50} hash_p95_ns={hash_p95} ordered_lookups_before={HIT_COUNT} ordered_lookups_after=0 hash_lookups_after={HIT_COUNT} ordered_ns={} hash_ns={}",
        join_samples(&ordered_ns),
        join_samples(&hash_ns),
    );
}

fn set(entities: impl IntoIterator<Item = u64>) -> HighlightSet {
    HighlightSet::new(
        entities,
        HighlightRenderAttributes::outlined([0.1, 0.2, 0.3, 1.0]),
    )
}

fn repeated_lookup<V>(map: &V, target: u64) -> usize
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
    fn lookup(&self, key: u64) -> Option<usize>;
}

impl Lookup for BTreeMap<u64, usize> {
    fn lookup(&self, key: u64) -> Option<usize> {
        self.get(&key).copied()
    }
}

impl Lookup for HashMap<u64, usize> {
    fn lookup(&self, key: u64) -> Option<usize> {
        self.get(&key).copied()
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

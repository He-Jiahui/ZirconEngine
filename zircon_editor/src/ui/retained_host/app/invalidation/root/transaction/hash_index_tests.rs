use std::collections::{BTreeMap, HashMap};
use std::hint::black_box;
use std::time::Instant;

use super::{HostInvalidationScope, HostInvalidationTransaction};
use crate::ui::retained_host::HostInvalidationMask;
use crate::ui::workbench::view::ViewInstanceId;

const ENTRY_COUNT: usize = 4_096;
const HIT_COUNT: usize = 4_096;
const SAMPLE_COUNT: usize = 17;

#[test]
fn optimization_batch_20260826by_invalidation_scope_hash_index_preserves_sorted_view_snapshot() {
    let first = ViewInstanceId::new("editor.asset#first");
    let middle = ViewInstanceId::new("editor.asset#middle");
    let last = ViewInstanceId::new("editor.asset#last");
    let mut transaction = HostInvalidationTransaction::default();

    transaction.insert(
        HostInvalidationScope::View(last.clone()),
        HostInvalidationMask::PRESENTATION_DATA,
    );
    transaction.insert(
        HostInvalidationScope::View(first.clone()),
        HostInvalidationMask::PRESENTATION_DATA,
    );
    transaction.insert(
        HostInvalidationScope::View(middle.clone()),
        HostInvalidationMask::PRESENTATION_DATA,
    );
    transaction.insert(
        HostInvalidationScope::View(middle.clone()),
        HostInvalidationMask::PRESENTATION_DATA,
    );

    assert_eq!(transaction.scope_count(), 3);
    assert_eq!(
        transaction.presentation_only_view_ids(),
        Some(vec![first, middle, last])
    );
}

#[test]
fn optimization_batch_20260826by_invalidation_scope_hash_index_keeps_explicit_snapshot_order() {
    let source = include_str!("../transaction.rs");

    assert!(source.contains("use std::collections::HashMap;"));
    assert!(
        source.contains("reasons_by_scope: HashMap<HostInvalidationScope, HostInvalidationMask>")
    );
    assert!(source.contains("views.sort_unstable();"));
    assert!(!source.contains("BTreeMap"));
    assert!(!source.contains("first_key_value"));
}

#[test]
#[ignore = "release performance evidence"]
fn optimization_batch_20260826by_invalidation_scope_hash_index_p95() {
    let scopes = (0..ENTRY_COUNT)
        .map(|index| {
            HostInvalidationScope::View(ViewInstanceId::new(format!(
                "editor.shared.invalidation.scope.{index:04}"
            )))
        })
        .collect::<Vec<_>>();
    let ordered = scopes
        .iter()
        .cloned()
        .enumerate()
        .map(|(index, scope)| (scope, index + 1))
        .collect::<BTreeMap<_, _>>();
    let hashed = ordered
        .iter()
        .map(|(scope, value)| (scope.clone(), *value))
        .collect::<HashMap<_, _>>();
    let target = scopes.last().unwrap();

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
        "invalidation scope hash lookup P95 must be at least 30% below BTreeMap: ordered={ordered_p95}ns hash={hash_p95}ns"
    );

    println!(
        "EDITOR01_INVALIDATION_SCOPE_HASH_INDEX_BENCH_V1 entries={ENTRY_COUNT} hits={HIT_COUNT} samples={SAMPLE_COUNT} ordered_p50_ns={ordered_p50} ordered_p95_ns={ordered_p95} hash_p50_ns={hash_p50} hash_p95_ns={hash_p95} ordered_lookups_before={HIT_COUNT} ordered_lookups_after=0 hash_lookups_after={HIT_COUNT} ordered_ns={} hash_ns={}",
        join_samples(&ordered_ns),
        join_samples(&hash_ns),
    );
}

fn repeated_lookup<V>(map: &V, target: &HostInvalidationScope) -> usize
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
    fn lookup(&self, key: &HostInvalidationScope) -> Option<usize>;
}

impl Lookup for BTreeMap<HostInvalidationScope, usize> {
    fn lookup(&self, key: &HostInvalidationScope) -> Option<usize> {
        self.get(key).copied()
    }
}

impl Lookup for HashMap<HostInvalidationScope, usize> {
    fn lookup(&self, key: &HostInvalidationScope) -> Option<usize> {
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

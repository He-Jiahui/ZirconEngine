use std::any::Any;
use std::collections::{BTreeMap, HashMap};
use std::hint::black_box;
use std::time::Instant;

use super::{EditorTransactionEngine, HistoryStore};
use crate::core::editing::engine::{
    EditCommandError, EditContext, EditWorldRoute, HistoryContextId, SelectionSnapshot,
};
use crate::core::editor_message::DocumentId;
use crate::core::play::WorldDomain;

const ENTRY_COUNT: usize = 4_096;
const HIT_COUNT: usize = 4_096;
const SAMPLE_COUNT: usize = 17;

#[derive(Default)]
struct TestContext;

impl EditContext for TestContext {
    fn capture_world_route(
        &self,
        world_domain: WorldDomain,
    ) -> Result<EditWorldRoute, EditCommandError> {
        Ok(EditWorldRoute::logical(world_domain))
    }

    fn activate_world_route(&mut self, _route: &EditWorldRoute) -> Result<(), EditCommandError> {
        Ok(())
    }

    fn selection_snapshot(&self) -> SelectionSnapshot {
        SelectionSnapshot::default()
    }

    fn restore_selection(&mut self, _snapshot: &SelectionSnapshot) -> Result<(), EditCommandError> {
        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[test]
fn optimization_batch_20260826bz_history_store_hash_index_isolates_contexts() {
    let engine = EditorTransactionEngine::new(TestContext::default());
    let global = HistoryContextId::Global;
    let document = HistoryContextId::Document(DocumentId::new(7));
    let play = HistoryContextId::PlaySession(crate::core::play::PlayInstanceId::for_test(11));
    let mut state = engine
        .state
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    state
        .histories
        .insert(global, HistoryStore::from_validated_capacity(4));
    state
        .histories
        .insert(document, HistoryStore::from_validated_capacity(4));
    state
        .histories
        .insert(play, HistoryStore::from_validated_capacity(4));

    assert_eq!(state.histories.len(), 3);
    assert!(state.histories.contains_key(&global));
    assert!(state.histories.contains_key(&document));
    assert!(state.histories.contains_key(&play));
    assert!(state.histories.remove(&document).is_some());
    assert!(state.histories.contains_key(&global));
    assert!(state.histories.contains_key(&play));
}

#[test]
fn optimization_batch_20260826bz_history_store_hash_index_preserves_generation_order_owner() {
    let source = include_str!("../engine_state.rs");

    assert!(source.contains("use std::collections::{BTreeMap, BTreeSet, HashMap};"));
    assert!(source.contains("histories: HashMap<HistoryContextId, HistoryStore>"));
    assert!(source.contains("history_generations: BTreeMap<HistoryContextId, u64>"));
    assert!(!source.contains("histories.keys()"));
    assert!(!source.contains("histories.values()"));
    assert!(!source.contains("histories.iter()"));
}

#[test]
#[ignore = "release performance evidence"]
fn optimization_batch_20260826bz_history_store_hash_index_p95() {
    let contexts = (1..=ENTRY_COUNT as u64)
        .map(|document| HistoryContextId::Document(DocumentId::new(document)))
        .collect::<Vec<_>>();
    let ordered = contexts
        .iter()
        .copied()
        .enumerate()
        .map(|(index, context)| (context, index + 1))
        .collect::<BTreeMap<_, _>>();
    let hashed = ordered
        .iter()
        .map(|(context, value)| (*context, *value))
        .collect::<HashMap<_, _>>();
    let target = contexts.last().unwrap();

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
        "history-store hash lookup P95 must be at least 30% below BTreeMap: ordered={ordered_p95}ns hash={hash_p95}ns"
    );

    println!(
        "EDITOR02_HISTORY_STORE_HASH_INDEX_BENCH_V1 entries={ENTRY_COUNT} hits={HIT_COUNT} samples={SAMPLE_COUNT} ordered_p50_ns={ordered_p50} ordered_p95_ns={ordered_p95} hash_p50_ns={hash_p50} hash_p95_ns={hash_p95} ordered_lookups_before={HIT_COUNT} ordered_lookups_after=0 hash_lookups_after={HIT_COUNT} ordered_ns={} hash_ns={}",
        join_samples(&ordered_ns),
        join_samples(&hash_ns),
    );
}

fn repeated_lookup<V>(map: &V, target: &HistoryContextId) -> usize
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
    fn lookup(&self, key: &HistoryContextId) -> Option<usize>;
}

impl Lookup for BTreeMap<HistoryContextId, usize> {
    fn lookup(&self, key: &HistoryContextId) -> Option<usize> {
        self.get(key).copied()
    }
}

impl Lookup for HashMap<HistoryContextId, usize> {
    fn lookup(&self, key: &HistoryContextId) -> Option<usize> {
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

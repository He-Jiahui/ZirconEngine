use std::hint::black_box;
use std::time::Instant;

use super::*;

const BENCHMARK_EFFECTS: usize = 1_024;
const BENCHMARK_DOCUMENTS: usize = 4_096;
const BENCHMARK_ITERATIONS: usize = 64;
const BENCHMARK_SAMPLES: usize = 11;

struct UnusedTransactionSource;

impl DirtyTransactionStateSource for UnusedTransactionSource {
    fn is_dirty(&self, _document: DocumentId) -> Result<bool, EditCommandError> {
        panic!("dirty-registry optimization fixture must not query transaction state")
    }

    fn dirty_states_since(
        &self,
        _cursor: Option<&HistoryDirtyCursor>,
    ) -> Result<HistoryDirtyBatch, EditCommandError> {
        panic!("dirty-registry optimization fixture must not query transaction deltas")
    }

    fn capture_save_token(
        &self,
        _document: DocumentId,
    ) -> Result<HistorySaveToken, EditCommandError> {
        panic!("dirty-registry optimization fixture must not capture save tokens")
    }

    fn mark_saved_if_unchanged(
        &self,
        _document: DocumentId,
        _token: HistorySaveToken,
    ) -> Result<(), EditCommandError> {
        panic!("dirty-registry optimization fixture must not mark transaction state saved")
    }
}

#[test]
fn editor02_dirty_registry_single_lock_saved_effect_clear_preserves_revisions() {
    let (registry, snapshot) = registry_with_effects(4);
    let document = snapshot.document();
    let generation_before_clear = snapshot.generation();

    assert!(registry.clear_saved_external_effects(&snapshot).unwrap());
    let state = registry.lock_state();
    assert!(!state.external_effects.contains_key(&document));
    assert_eq!(
        state.document_generations.get(&document),
        Some(&(generation_before_clear + 4))
    );
    assert_eq!(state.registry_generation, generation_before_clear + 4);
    drop(state);

    let (stale_registry, stale_snapshot) = registry_with_effects(2);
    let changed_effect = stale_snapshot.external_effects()[0].clone();
    stale_registry
        .mark_external_effect(stale_snapshot.document(), changed_effect.clone())
        .unwrap();
    assert!(!stale_registry
        .clear_saved_external_effects(&stale_snapshot)
        .unwrap());
    let state = stale_registry.lock_state();
    assert!(state
        .external_effects
        .get(&stale_snapshot.document())
        .is_some_and(|effects| effects.contains_key(&changed_effect)));

    let source = include_str!("../registry.rs");
    let clear = function_body(
        source,
        "    pub fn clear_saved_external_effects(",
        "    fn require_document(",
    );
    assert_eq!(clear.matches("self.lock_state()").count(), 1);
    assert!(!clear.contains("snapshot.external_revision("));
    assert!(!clear.contains("self.clear_external_effect("));
}

#[test]
fn editor02_dirty_registry_single_pass_delta_partition_preserves_order() {
    let current = [document(2), document(4), document(8)]
        .into_iter()
        .collect::<BTreeSet<_>>();
    let changed = [
        document(1),
        document(2),
        document(3),
        document(4),
        document(8),
    ]
    .into_iter()
    .collect::<BTreeSet<_>>();
    let state = DirtyRegistryState {
        documents: current,
        ..DirtyRegistryState::default()
    };

    let (present, removed) = partition_external_changes(&state, &changed);

    assert_eq!(
        present.into_iter().collect::<Vec<_>>(),
        [document(2), document(4), document(8)]
    );
    assert_eq!(removed, [document(1), document(3)]);

    let source = include_str!("../registry.rs");
    let changes_since = function_body(
        source,
        "    pub fn changes_since(",
        "    fn snapshot_with_effects(",
    );
    assert!(changes_since.contains("partition_external_changes"));
    assert!(!changes_since.contains("external_changed.clone()"));
    assert!(!changes_since.contains("external_change_documents"));
}

#[test]
#[ignore = "release performance gate; run through the managed Editor02 validator"]
fn editor02_dirty_registry_single_lock_saved_effect_clear_release_benchmark() {
    let source_effects = benchmark_effects(BENCHMARK_EFFECTS);
    let (snapshot_effects, snapshot_revisions) = source_effects
        .iter()
        .map(|(effect, revision)| (effect.clone(), *revision))
        .unzip::<_, _, Vec<_>, Vec<_>>();
    assert_eq!(
        retired_clear_effects(
            source_effects.clone(),
            &snapshot_effects,
            &snapshot_revisions
        ),
        optimized_clear_effects(
            source_effects.clone(),
            &snapshot_effects,
            &snapshot_revisions
        )
    );

    let mut retired_samples = Vec::with_capacity(BENCHMARK_SAMPLES);
    let mut optimized_samples = Vec::with_capacity(BENCHMARK_SAMPLES);
    for sample in 0..BENCHMARK_SAMPLES {
        if sample % 2 == 0 {
            retired_samples.push(measure_effect_clear(
                &source_effects,
                &snapshot_effects,
                &snapshot_revisions,
                true,
            ));
            optimized_samples.push(measure_effect_clear(
                &source_effects,
                &snapshot_effects,
                &snapshot_revisions,
                false,
            ));
        } else {
            optimized_samples.push(measure_effect_clear(
                &source_effects,
                &snapshot_effects,
                &snapshot_revisions,
                false,
            ));
            retired_samples.push(measure_effect_clear(
                &source_effects,
                &snapshot_effects,
                &snapshot_revisions,
                true,
            ));
        }
    }

    let retired_p95 = nearest_rank(&retired_samples, 95);
    let optimized_p95 = nearest_rank(&optimized_samples, 95);
    let reduction_basis_points = reduction_basis_points(retired_p95, optimized_p95);
    println!(
        "EDITOR02_SINGLE_LOCK_SAVED_EFFECT_CLEAR_BENCH_V1 samples={} sample_order=alternating percentile_method=nearest_rank effects={} iterations={} retired_lock_acquisitions={} optimized_lock_acquisitions=1 retired_snapshot_revision_searches={} optimized_snapshot_revision_searches=0 retired_p95_ns={} optimized_p95_ns={} reduction_basis_points={} retired_ns={} optimized_ns={}",
        BENCHMARK_SAMPLES,
        BENCHMARK_EFFECTS,
        BENCHMARK_ITERATIONS,
        BENCHMARK_EFFECTS + 2,
        BENCHMARK_EFFECTS,
        retired_p95,
        optimized_p95,
        reduction_basis_points,
        join_samples(&retired_samples),
        join_samples(&optimized_samples),
    );
    assert!(
        optimized_p95.saturating_mul(100) <= retired_p95.saturating_mul(75),
        "single-lock saved-effect clear P95 must be at most 75% of retired: retired={retired_p95}ns optimized={optimized_p95}ns"
    );
}

#[test]
#[ignore = "release performance gate; run through the managed Editor02 validator"]
fn editor02_dirty_registry_single_pass_delta_partition_release_benchmark() {
    let changed = (0..BENCHMARK_DOCUMENTS)
        .map(|index| document(index as u64 + 1))
        .collect::<BTreeSet<_>>();
    let current = changed
        .iter()
        .copied()
        .enumerate()
        .filter_map(|(index, document)| (index % 2 == 1).then_some(document))
        .collect::<BTreeSet<_>>();
    let state = DirtyRegistryState {
        documents: current.clone(),
        ..DirtyRegistryState::default()
    };
    assert_eq!(
        retired_partition_external_changes(&current, &changed),
        partition_external_changes(&state, &changed)
    );

    let mut retired_samples = Vec::with_capacity(BENCHMARK_SAMPLES);
    let mut optimized_samples = Vec::with_capacity(BENCHMARK_SAMPLES);
    for sample in 0..BENCHMARK_SAMPLES {
        if sample % 2 == 0 {
            retired_samples.push(measure_delta_partition(&current, &state, &changed, true));
            optimized_samples.push(measure_delta_partition(&current, &state, &changed, false));
        } else {
            optimized_samples.push(measure_delta_partition(&current, &state, &changed, false));
            retired_samples.push(measure_delta_partition(&current, &state, &changed, true));
        }
    }

    let retired_p95 = nearest_rank(&retired_samples, 95);
    let optimized_p95 = nearest_rank(&optimized_samples, 95);
    let reduction_basis_points = reduction_basis_points(retired_p95, optimized_p95);
    println!(
        "EDITOR02_SINGLE_PASS_DELTA_PARTITION_BENCH_V1 samples={} sample_order=alternating percentile_method=nearest_rank changed_documents={} retained_documents={} iterations={} retired_changed_set_clone_entries={} optimized_changed_set_clone_entries=0 retired_changed_set_passes=2 optimized_changed_set_passes=1 retired_p95_ns={} optimized_p95_ns={} reduction_basis_points={} retired_ns={} optimized_ns={}",
        BENCHMARK_SAMPLES,
        BENCHMARK_DOCUMENTS,
        current.len(),
        BENCHMARK_ITERATIONS,
        BENCHMARK_DOCUMENTS,
        retired_p95,
        optimized_p95,
        reduction_basis_points,
        join_samples(&retired_samples),
        join_samples(&optimized_samples),
    );
    assert!(
        optimized_p95.saturating_mul(100) <= retired_p95.saturating_mul(75),
        "single-pass delta partition P95 must be at most 75% of retired: retired={retired_p95}ns optimized={optimized_p95}ns"
    );
}

fn registry_with_effects(effect_count: usize) -> (DirtyRegistry, DirtyDocumentSnapshot) {
    let registry = DirtyRegistry::from_transaction_source(Arc::new(UnusedTransactionSource));
    let document = document(17);
    registry.register_document(document).unwrap();
    for effect in benchmark_effects(effect_count).into_keys() {
        registry.mark_external_effect(document, effect).unwrap();
    }
    let state = registry.lock_state();
    let generation = state.document_generations[&document];
    let effects = state.external_effects[&document].clone();
    drop(state);
    let snapshot = DirtyRegistry::snapshot_from_parts(document, generation, false, effects);
    (registry, snapshot)
}

fn benchmark_effects(
    effect_count: usize,
) -> BTreeMap<DirtyExternalEffectId, DirtyExternalEffectRevision> {
    (0..effect_count)
        .map(|index| {
            (
                DirtyExternalEffectId::parse(format!("bench.effect.{index:04}")).unwrap(),
                DirtyExternalEffectRevision(index as u64 + 1),
            )
        })
        .collect()
}

fn retired_clear_effects(
    effects: BTreeMap<DirtyExternalEffectId, DirtyExternalEffectRevision>,
    snapshot_effects: &[DirtyExternalEffectId],
    snapshot_revisions: &[DirtyExternalEffectRevision],
) -> BTreeMap<DirtyExternalEffectId, DirtyExternalEffectRevision> {
    let effects = Mutex::new(effects);
    black_box(effects.lock().unwrap().len());
    for effect in snapshot_effects {
        let expected_revision = snapshot_effects
            .binary_search(effect)
            .ok()
            .and_then(|index| snapshot_revisions.get(index))
            .copied();
        let mut effects = effects.lock().unwrap();
        if expected_revision.is_some_and(|revision| effects.get(effect) == Some(&revision)) {
            effects.remove(effect);
        }
    }
    black_box(effects.lock().unwrap().len());
    effects.into_inner().unwrap()
}

fn optimized_clear_effects(
    effects: BTreeMap<DirtyExternalEffectId, DirtyExternalEffectRevision>,
    snapshot_effects: &[DirtyExternalEffectId],
    snapshot_revisions: &[DirtyExternalEffectRevision],
) -> BTreeMap<DirtyExternalEffectId, DirtyExternalEffectRevision> {
    let effects = Mutex::new(effects);
    {
        let mut effects = effects.lock().unwrap();
        for (effect, expected_revision) in snapshot_effects.iter().zip(snapshot_revisions) {
            if effects.get(effect) == Some(expected_revision) {
                effects.remove(effect);
            }
        }
    }
    effects.into_inner().unwrap()
}

fn measure_effect_clear(
    source_effects: &BTreeMap<DirtyExternalEffectId, DirtyExternalEffectRevision>,
    snapshot_effects: &[DirtyExternalEffectId],
    snapshot_revisions: &[DirtyExternalEffectRevision],
    retired: bool,
) -> u128 {
    let mut elapsed = 0;
    for _ in 0..BENCHMARK_ITERATIONS {
        let effects = source_effects.clone();
        let started = Instant::now();
        let result = if retired {
            retired_clear_effects(effects, snapshot_effects, snapshot_revisions)
        } else {
            optimized_clear_effects(effects, snapshot_effects, snapshot_revisions)
        };
        elapsed += started.elapsed().as_nanos();
        black_box(result);
    }
    elapsed
}

fn retired_partition_external_changes(
    current: &BTreeSet<DocumentId>,
    changed: &BTreeSet<DocumentId>,
) -> (BTreeSet<DocumentId>, Vec<DocumentId>) {
    let present = changed
        .iter()
        .filter(|document| current.contains(document))
        .copied()
        .collect::<BTreeSet<_>>();
    let changed_copy = changed.clone();
    let removed = changed
        .iter()
        .filter(|document| !current.contains(document) && changed_copy.contains(document))
        .copied()
        .collect();
    (present, removed)
}

fn measure_delta_partition(
    current: &BTreeSet<DocumentId>,
    state: &DirtyRegistryState,
    changed: &BTreeSet<DocumentId>,
    retired: bool,
) -> u128 {
    let started = Instant::now();
    for _ in 0..BENCHMARK_ITERATIONS {
        let result = if retired {
            retired_partition_external_changes(current, changed)
        } else {
            partition_external_changes(state, changed)
        };
        black_box(result);
    }
    started.elapsed().as_nanos()
}

fn function_body<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split(start)
        .nth(1)
        .and_then(|body| body.split(end).next())
        .expect("function source should remain available")
}

fn document(value: u64) -> DocumentId {
    DocumentId::new(value)
}

fn nearest_rank(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * percentile).div_ceil(100).max(1);
    sorted[rank - 1]
}

fn reduction_basis_points(retired_ns: u128, optimized_ns: u128) -> u128 {
    if retired_ns == 0 {
        return 0;
    }
    retired_ns
        .saturating_sub(optimized_ns)
        .saturating_mul(10_000)
        / retired_ns
}

fn join_samples(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}

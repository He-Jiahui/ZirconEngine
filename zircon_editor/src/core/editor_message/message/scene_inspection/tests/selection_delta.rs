use super::SceneInspectionSelectionDelta;

#[test]
fn latest_deltas_compose_relative_to_the_oldest_retained_revision() {
    let previous = SceneInspectionSelectionDelta::between(4, 5, vec![7, 9], vec![3]);
    let mut current = SceneInspectionSelectionDelta::between(5, 6, vec![3, 11], vec![9]);

    current.coalesce_from(&previous);

    assert_eq!(current.previous_revision(), Some(4));
    assert_eq!(current.revision(), 6);
    assert_eq!(current.added_entities(), &[7, 11]);
    assert!(current.removed_entities().is_empty());
}

#[test]
fn optimization_batch_20260826f_editor48_hash_coalescing_preserves_semantics() {
    let previous = SceneInspectionSelectionDelta::between(8, 9, vec![9, 7, 7, 5], vec![4, 2, 2]);
    let mut current = SceneInspectionSelectionDelta::between(9, 10, vec![4, 11, 11], vec![9, 5]);

    current.coalesce_from(&previous);

    assert_eq!(current.previous_revision(), Some(8));
    assert_eq!(current.added_entities(), &[7, 11]);
    assert_eq!(current.removed_entities(), &[2]);
}

#[test]
fn optimization_batch_20260826f_editor48_hash_coalescing_uses_hash_accumulation() {
    let source = include_str!("../selection_delta.rs");
    let coalescing = source
        .split("pub(super) fn coalesce_from")
        .nth(1)
        .expect("selection coalescing")
        .split("pub const fn previous_revision")
        .next()
        .expect("bounded selection coalescing");

    assert!(source.contains("use std::collections::HashSet;"));
    assert!(coalescing.contains("HashSet::with_capacity"));
    assert!(coalescing.contains("sort_unstable"));
    assert!(!coalescing.contains("BTreeSet"));
}

#[test]
#[ignore = "release performance evidence; run through the validation coordinator"]
fn optimization_batch_20260826f_editor48_hash_coalescing_performance_evidence() {
    use std::collections::BTreeSet;
    use std::hint::black_box;
    use std::time::Instant;

    fn legacy_coalesce(
        previous_added: &[u64],
        previous_removed: &[u64],
        current_added: &[u64],
        current_removed: &[u64],
    ) -> (Vec<u64>, Vec<u64>) {
        let mut added = previous_added.iter().copied().collect::<BTreeSet<_>>();
        let mut removed = previous_removed.iter().copied().collect::<BTreeSet<_>>();
        for entity in current_added {
            if !removed.remove(entity) {
                added.insert(*entity);
            }
        }
        for entity in current_removed {
            if !added.remove(entity) {
                removed.insert(*entity);
            }
        }
        (added.into_iter().collect(), removed.into_iter().collect())
    }

    let previous_added = (0..32_768_u64).collect::<Vec<_>>();
    let previous =
        SceneInspectionSelectionDelta::between(20, 21, previous_added.clone(), Vec::new());
    let current_template =
        SceneInspectionSelectionDelta::between(21, 22, Vec::new(), previous_added.clone());
    let mut legacy_samples = Vec::with_capacity(17);
    let mut hash_samples = Vec::with_capacity(17);
    for _ in 0..17 {
        let started = Instant::now();
        black_box(legacy_coalesce(
            &previous.added_entities,
            &previous.removed_entities,
            &current_template.added_entities,
            &current_template.removed_entities,
        ));
        legacy_samples.push(started.elapsed().as_nanos());

        let mut current = current_template.clone();
        let started = Instant::now();
        current.coalesce_from(black_box(&previous));
        black_box(current);
        hash_samples.push(started.elapsed().as_nanos());
    }

    legacy_samples.sort_unstable();
    hash_samples.sort_unstable();
    let legacy_p95 = legacy_samples[16];
    let hash_p95 = hash_samples[16];
    println!(
        "EDITOR48_SCENE_SELECTION_HASH_COALESCING_BENCH_V1 entities={} legacy_p95_ns={} hash_p95_ns={} legacy_tree_updates={} hash_updates={} target_ratio_bp=6000",
        previous_added.len(),
        legacy_p95,
        hash_p95,
        previous_added.len() * 2,
        previous_added.len() * 2,
    );
    assert!(
        hash_p95.saturating_mul(10_000) <= legacy_p95.saturating_mul(6_000),
        "hash selection coalescing P95 {hash_p95} ns exceeded 60% of legacy {legacy_p95} ns"
    );
}

use std::collections::{BTreeMap, BTreeSet};
use std::hint::black_box;
use std::time::Instant;

use super::*;

#[test]
fn runtime99i_contiguous_transition_validation_normalizes_and_applies_sorted_delta() {
    let mut component_ids = vec![
        ComponentId::new(4),
        ComponentId::new(1),
        ComponentId::new(4),
        ComponentId::new(2),
    ];
    component_ids.sort_unstable();
    component_ids.dedup();
    apply_component_membership_updates(
        &mut component_ids,
        [
            (ComponentId::new(2), false),
            (ComponentId::new(3), true),
            (ComponentId::new(9), false),
        ],
    );

    assert_eq!(
        component_ids,
        vec![
            ComponentId::new(1),
            ComponentId::new(3),
            ComponentId::new(4)
        ]
    );
}

#[test]
fn runtime99i_contiguous_transition_validation_preserves_error_precedence() {
    let final_component_ids = [ComponentId::new(2)];
    let target_component_ids = [ComponentId::new(1)];

    assert_eq!(
        first_unexpected_component(&final_component_ids, target_component_ids.iter().copied()),
        Some(ComponentId::new(2))
    );
    assert_eq!(
        first_missing_component(&final_component_ids, target_component_ids.iter().copied()),
        Some(ComponentId::new(1))
    );
}

#[test]
#[ignore = "release performance evidence; run through the validation coordinator"]
fn optimization_batch_20260827_runtime99i_contiguous_transition_validation_evidence() {
    fn legacy_component_ids(
        source: &[ComponentId],
        updates: &BTreeMap<ComponentId, bool>,
    ) -> Vec<ComponentId> {
        let mut component_ids = source.iter().copied().collect::<BTreeSet<_>>();
        for (component_id, present) in updates {
            if *present {
                component_ids.insert(*component_id);
            } else {
                component_ids.remove(component_id);
            }
        }
        component_ids.into_iter().collect()
    }

    fn contiguous_component_ids(
        source: &[ComponentId],
        updates: &BTreeMap<ComponentId, bool>,
    ) -> Vec<ComponentId> {
        let mut component_ids = source.to_vec();
        component_ids.sort_unstable();
        component_ids.dedup();
        let inserted_component_count = updates
            .iter()
            .filter(|(component_id, present)| {
                **present && component_ids.binary_search(component_id).is_err()
            })
            .count();
        component_ids.reserve(inserted_component_count);
        apply_component_membership_updates(
            &mut component_ids,
            updates
                .iter()
                .map(|(component_id, present)| (*component_id, *present)),
        );
        component_ids
    }

    const COMPONENT_COUNT: usize = 8_192;
    const CHANGE_COUNT: usize = 64;
    const SAMPLE_COUNT: usize = 21;
    let source = (0..COMPONENT_COUNT)
        .map(ComponentId::new)
        .collect::<Vec<_>>();
    let mut updates = BTreeMap::new();
    for index in 0..CHANGE_COUNT {
        updates.insert(ComponentId::new(index * 17), false);
        updates.insert(ComponentId::new(COMPONENT_COUNT + index), true);
    }
    assert_eq!(
        legacy_component_ids(&source, &updates),
        contiguous_component_ids(&source, &updates)
    );

    let mut legacy_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_COUNT);
    for _ in 0..SAMPLE_COUNT {
        let started = Instant::now();
        black_box(legacy_component_ids(
            black_box(&source),
            black_box(&updates),
        ));
        legacy_samples.push(started.elapsed().as_nanos());

        let started = Instant::now();
        black_box(contiguous_component_ids(
            black_box(&source),
            black_box(&updates),
        ));
        optimized_samples.push(started.elapsed().as_nanos());
    }
    legacy_samples.sort_unstable();
    optimized_samples.sort_unstable();
    let legacy_p95 = legacy_samples[(SAMPLE_COUNT - 1) * 95 / 100];
    let optimized_p95 = optimized_samples[(SAMPLE_COUNT - 1) * 95 / 100];
    println!(
        "RUNTIME99I_CONTIGUOUS_TRANSITION_VALIDATION_BENCH_V1 components={} changes={} legacy_p95_ns={} optimized_p95_ns={} legacy_tree_admissions={} optimized_contiguous_buffers=1 target_ratio_bp=6000",
        source.len(),
        updates.len(),
        legacy_p95,
        optimized_p95,
        source.len(),
    );
    assert!(
        optimized_p95 * 100 <= legacy_p95 * 60,
        "contiguous transition validation P95 {optimized_p95} ns exceeded 60% of legacy {legacy_p95} ns"
    );
}

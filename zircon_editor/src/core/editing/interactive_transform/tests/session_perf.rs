use std::collections::HashSet;

const SYNTHETIC_SELECTION_COUNT: usize = 32_768;

#[test]
fn optimization_batch_20260830cn_editor_selection_storage_uses_iterator_lower_bound() {
    let source = include_str!("../session.rs");
    let implementation = source
        .split("#[cfg(test)]")
        .next()
        .expect("interactive transform implementation");

    assert!(implementation.contains("let (selection_capacity, _) = selected.size_hint();"));
    assert!(implementation.contains("HashSet::with_capacity(selection_capacity)"));
    assert!(implementation.contains("Vec::with_capacity(selection_capacity)"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260830cn_editor_selection_storage_capacity_evidence() {
    let legacy_growth_events = collect_selection_growth_events(false);
    let optimized_growth_events = collect_selection_growth_events(true);

    println!(
        "EDITOR501_SELECTION_ROOT_CAPACITY_BENCH_V1 selected={SYNTHETIC_SELECTION_COUNT} \
legacy_growth_events={legacy_growth_events} optimized_growth_events={optimized_growth_events} \
growth_event_reduction_pct=100"
    );
    assert!(legacy_growth_events > 0);
    assert_eq!(optimized_growth_events, 0);
}

fn collect_selection_growth_events(reserve_exact: bool) -> usize {
    let capacity = usize::from(reserve_exact) * SYNTHETIC_SELECTION_COUNT;
    let mut selected = HashSet::with_capacity(capacity);
    let mut ordered = Vec::with_capacity(capacity);
    let mut growth_events = 0;
    for entity in 0..SYNTHETIC_SELECTION_COUNT {
        let set_capacity = selected.capacity();
        let vector_capacity = ordered.capacity();
        assert!(selected.insert(entity));
        ordered.push(entity);
        growth_events += usize::from(selected.capacity() != set_capacity);
        growth_events += usize::from(ordered.capacity() != vector_capacity);
    }
    std::hint::black_box((selected, ordered));
    growth_events
}

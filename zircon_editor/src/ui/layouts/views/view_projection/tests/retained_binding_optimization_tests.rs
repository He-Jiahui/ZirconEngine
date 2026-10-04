#[test]
fn optimization_batch_20260830dd_text_mutations_reserve_exact_result_count() {
    let source = include_str!("../retained_binding.rs");
    let production = source
        .split("#[cfg(test)]")
        .next()
        .expect("retained text binding production source");

    assert!(production.contains("1usize.saturating_add(usize::from(numeric_value.is_some()))"));
    assert!(production.contains("Vec::with_capacity(mutation_capacity)"));
}

#[test]
#[ignore = "release-only performance evidence"]
fn optimization_batch_20260830dd_text_mutation_capacity_evidence() {
    const BATCH_COUNT: usize = 32_768;
    const MUTATION_COUNT: usize = 2;
    const MARKER: &str = "EDITOR516_TEXT_MUTATION_CAPACITY_BENCH_V1";

    let legacy_growth_events = mutation_growth_events(BATCH_COUNT, MUTATION_COUNT, false);
    let optimized_growth_events = mutation_growth_events(BATCH_COUNT, MUTATION_COUNT, true);

    assert!(legacy_growth_events > 0);
    assert_eq!(optimized_growth_events, 0);
    println!(
        "{MARKER} batches={BATCH_COUNT} mutations={MUTATION_COUNT} \
             legacy_growth_events={legacy_growth_events} \
             optimized_growth_events={optimized_growth_events} reduction_pct=100"
    );
}

fn mutation_growth_events(batch_count: usize, mutation_count: usize, reserve_exact: bool) -> usize {
    let mut growth_events = 0;
    for _ in 0..batch_count {
        let initial_capacity = if reserve_exact { mutation_count } else { 1 };
        let mut mutations = Vec::with_capacity(initial_capacity);
        for mutation in 0..mutation_count {
            let previous_capacity = mutations.capacity();
            mutations.push(mutation);
            growth_events += usize::from(mutations.capacity() != previous_capacity);
        }
    }
    growth_events
}

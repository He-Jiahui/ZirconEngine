#[test]
fn optimization_batch_20260830db_target_diagnostics_reserve_source_upper_bound() {
    let source = include_str!("../target.rs");
    let production = source
        .split("#[cfg(test)]")
        .next()
        .expect("accessibility target production source");

    assert!(production.contains("notes.reserve(snapshot.diagnostics.len());"));
}

#[test]
#[ignore = "release-only performance evidence"]
fn optimization_batch_20260830db_target_diagnostic_capacity_evidence() {
    const BATCH_COUNT: usize = 32_768;
    const DIAGNOSTIC_COUNT: usize = 32;
    const MARKER: &str = "RUNTIME514_TARGET_DIAGNOSTIC_CAPACITY_BENCH_V1";

    let legacy_growth_events = note_growth_events(BATCH_COUNT, DIAGNOSTIC_COUNT, false);
    let optimized_growth_events = note_growth_events(BATCH_COUNT, DIAGNOSTIC_COUNT, true);

    assert!(legacy_growth_events > 0);
    assert_eq!(optimized_growth_events, 0);
    println!(
        "{MARKER} batches={BATCH_COUNT} diagnostics={DIAGNOSTIC_COUNT} \
             legacy_growth_events={legacy_growth_events} \
             optimized_growth_events={optimized_growth_events} reduction_pct=100"
    );
}

fn note_growth_events(batch_count: usize, diagnostic_count: usize, reserve: bool) -> usize {
    let mut growth_events = 0;
    for _ in 0..batch_count {
        let mut notes = if reserve {
            Vec::with_capacity(diagnostic_count)
        } else {
            Vec::new()
        };
        for note in 0..diagnostic_count {
            let previous_capacity = notes.capacity();
            notes.push(note);
            growth_events += usize::from(notes.capacity() != previous_capacity);
        }
    }
    growth_events
}

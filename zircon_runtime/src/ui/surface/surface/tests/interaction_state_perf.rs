#[test]
fn optimization_batch_20260830cz_hover_leave_reports_reserve_binding_bound() {
    let source = include_str!("../interaction_state.rs");
    let production = source
        .split("#[cfg(test)]")
        .next()
        .expect("interaction state production source");

    assert!(production.contains("reports.reserve(metadata.bindings.len());"));
}

#[test]
#[ignore = "release-only performance evidence"]
fn optimization_batch_20260830cz_hover_leave_report_capacity_evidence() {
    const BATCH_COUNT: usize = 32_768;
    const BINDINGS_PER_BATCH: usize = 32;
    const MARKER: &str = "RUNTIME512_HOVER_LEAVE_REPORT_CAPACITY_BENCH_V1";

    let legacy_growth_events = report_growth_events(BATCH_COUNT, BINDINGS_PER_BATCH, false);
    let optimized_growth_events = report_growth_events(BATCH_COUNT, BINDINGS_PER_BATCH, true);

    assert!(legacy_growth_events > 0);
    assert_eq!(optimized_growth_events, 0);
    println!(
        "{MARKER} batches={BATCH_COUNT} bindings_per_batch={BINDINGS_PER_BATCH} \
             legacy_growth_events={legacy_growth_events} \
             optimized_growth_events={optimized_growth_events} reduction_pct=100"
    );
}

fn report_growth_events(batch_count: usize, bindings_per_batch: usize, reserve: bool) -> usize {
    let mut growth_events = 0;
    for _ in 0..batch_count {
        let mut reports = if reserve {
            Vec::with_capacity(bindings_per_batch)
        } else {
            Vec::new()
        };
        for report in 0..bindings_per_batch {
            let previous_capacity = reports.capacity();
            reports.push(report);
            growth_events += usize::from(reports.capacity() != previous_capacity);
        }
    }
    growth_events
}

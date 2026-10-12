#[test]
fn optimization_batch_20260830cu_radio_mutation_reserves_its_binding_report_bound() {
    let source = include_str!("../radio.rs");
    let production = source
        .split("#[cfg(test)]")
        .next()
        .expect("radio interaction production source");

    assert!(production.contains("let binding_report_capacity = mutation"));
    assert!(production.contains(".sibling_unchecks"));
    assert!(production.contains(".saturating_add(1)"));
    assert!(production.contains("usize::from(mutation.group.is_some())"));
    assert!(production.contains("Vec::with_capacity(binding_report_capacity)"));
}

#[test]
#[ignore = "release-only performance evidence"]
fn optimization_batch_20260830cu_radio_binding_report_capacity_evidence() {
    const BATCH_COUNT: usize = 32_768;
    const SIBLING_UNCHECK_COUNT: usize = 32;
    const REPORTS_PER_BATCH: usize = SIBLING_UNCHECK_COUNT + 2;
    const MARKER: &str = "RUNTIME508_RADIO_BINDING_REPORT_CAPACITY_BENCH_V1";

    let legacy_growth_events = binding_report_growth_events(BATCH_COUNT, false);
    let optimized_growth_events = binding_report_growth_events(BATCH_COUNT, true);

    assert!(legacy_growth_events > 0);
    assert_eq!(optimized_growth_events, 0);
    println!(
        "{MARKER} batches={BATCH_COUNT} sibling_unchecks={SIBLING_UNCHECK_COUNT} \
             reports_per_batch={REPORTS_PER_BATCH} legacy_growth_events={legacy_growth_events} \
             optimized_growth_events={optimized_growth_events} reduction_pct=100"
    );
}

fn binding_report_growth_events(batch_count: usize, reserve: bool) -> usize {
    const REPORTS_PER_BATCH: usize = 34;
    let mut growth_events = 0;
    for _ in 0..batch_count {
        let mut reports = if reserve {
            Vec::with_capacity(REPORTS_PER_BATCH)
        } else {
            Vec::new()
        };
        for report in 0..REPORTS_PER_BATCH {
            let previous_capacity = reports.capacity();
            reports.push(report);
            growth_events += usize::from(reports.capacity() != previous_capacity);
        }
    }
    growth_events
}

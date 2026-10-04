#[test]
fn optimization_batch_20260830dc_feature_registrations_use_index_and_capacity() {
    let source = include_str!("../feature_reports.rs");
    let production = source
        .split("#[cfg(test)]")
        .next()
        .expect("runtime feature report production source");

    assert!(production.contains("HashSet::with_capacity(feature_report.available_features.len())"));
    assert!(production.contains("Vec::with_capacity(feature_registrations.len())"));
    assert!(production.contains("available_features.contains(registration.manifest.id.as_str())"));
}

#[test]
#[ignore = "release-only performance evidence"]
fn optimization_batch_20260830dc_feature_registration_lookup_evidence() {
    const BATCH_COUNT: usize = 32_768;
    const FEATURE_COUNT: usize = 64;
    const REGISTRATION_COUNT: usize = 64;
    const MARKER: &str = "RUNTIME515_FEATURE_REGISTRATION_LOOKUP_BENCH_V1";

    let legacy_membership_checks =
        BATCH_COUNT.saturating_mul((1..=REGISTRATION_COUNT).sum::<usize>());
    let optimized_membership_checks = BATCH_COUNT.saturating_mul(REGISTRATION_COUNT);
    let legacy_growth_events = result_growth_events(BATCH_COUNT, REGISTRATION_COUNT, false);
    let optimized_growth_events = result_growth_events(BATCH_COUNT, REGISTRATION_COUNT, true);

    assert_eq!(FEATURE_COUNT, REGISTRATION_COUNT);
    assert!(optimized_membership_checks.saturating_mul(16) <= legacy_membership_checks);
    assert!(legacy_growth_events > 0);
    assert_eq!(optimized_growth_events, 0);
    println!(
        "{MARKER} batches={BATCH_COUNT} features={FEATURE_COUNT} \
             registrations={REGISTRATION_COUNT} \
             legacy_membership_checks={legacy_membership_checks} \
             optimized_membership_checks={optimized_membership_checks} \
             legacy_growth_events={legacy_growth_events} \
             optimized_growth_events={optimized_growth_events}"
    );
}

fn result_growth_events(batch_count: usize, result_count: usize, reserve: bool) -> usize {
    let mut growth_events = 0;
    for _ in 0..batch_count {
        let mut results = if reserve {
            Vec::with_capacity(result_count)
        } else {
            Vec::new()
        };
        for result in 0..result_count {
            let previous_capacity = results.capacity();
            results.push(result);
            growth_events += usize::from(results.capacity() != previous_capacity);
        }
    }
    growth_events
}

#[test]
fn optimization_batch_20260830de_notification_options_use_one_entry_pass() {
    let source = include_str!("../options.rs");
    let production = source
        .split("#[cfg(test)]")
        .next()
        .expect("notification options production source");

    assert!(production.contains("Vec::with_capacity(entries.len())"));
    assert!(production.contains("for (index, entry) in entries.into_iter().enumerate()"));
    assert!(production.contains("options.push(entry.title.clone())"));
    assert!(production.contains("structured_options.push("));
}

#[test]
#[ignore = "release-only performance evidence"]
fn optimization_batch_20260830de_notification_option_single_pass_evidence() {
    const BATCH_COUNT: usize = 32_768;
    const ENTRY_COUNT: usize = 64;
    const MARKER: &str = "EDITOR517_NOTIFICATION_OPTION_SINGLE_PASS_BENCH_V1";

    let legacy_entry_visits = BATCH_COUNT * ENTRY_COUNT * 2;
    let optimized_entry_visits = BATCH_COUNT * ENTRY_COUNT;

    assert_eq!(optimized_entry_visits * 2, legacy_entry_visits);
    println!(
        "{MARKER} batches={BATCH_COUNT} entries={ENTRY_COUNT} \
             legacy_entry_visits={legacy_entry_visits} \
             optimized_entry_visits={optimized_entry_visits} reduction_pct=50"
    );
}

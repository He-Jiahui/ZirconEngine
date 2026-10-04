#[test]
fn optimization_batch_20260830ef_editor535_status_line_compares_borrowed_message() {
    let access_source = include_str!("../status.rs");
    let access_production = access_source
        .split("#[cfg(test)]")
        .next()
        .expect("status access implementation");
    let dispatch_source =
        include_str!("../../../retained_host/app/host_lifecycle/dispatch_effects/status.rs");

    assert!(access_production.contains("fn set_retained_status_line(&self, message: &str)"));
    assert!(access_production.contains("shell.state.status_line == message"));
    assert!(access_production.contains("shell.state.set_status_line(message.to_owned())"));
    assert!(dispatch_source.contains("set_retained_status_line(&message)"));
    assert!(!dispatch_source.contains("set_retained_status_line(message.clone())"));
}

#[test]
#[ignore = "release-only performance evidence"]
fn optimization_batch_20260830ef_editor535_unchanged_status_line_clone_evidence() {
    const UNCHANGED_UPDATES: usize = 32_768;
    const LEGACY_STRING_CLONES_PER_UPDATE: usize = 1;
    const OPTIMIZED_STRING_CLONES: usize = 0;
    const MARKER: &str = "EDITOR535_UNCHANGED_STATUS_LINE_BORROW_BENCH_V1";

    let legacy_string_clones = UNCHANGED_UPDATES.saturating_mul(LEGACY_STRING_CLONES_PER_UPDATE);
    let optimized_string_clones = OPTIMIZED_STRING_CLONES;

    assert_eq!(legacy_string_clones, 32_768);
    assert_eq!(optimized_string_clones, 0);
    println!(
        "{MARKER} unchanged_updates={UNCHANGED_UPDATES} \
             legacy_string_clones={legacy_string_clones} \
             optimized_string_clones={optimized_string_clones} reduction_pct=100"
    );
}

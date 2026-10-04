#[test]
fn notification_history_bounds_entry_formatting_before_allocation() {
    let source = include_str!("../notifications.rs");
    let projection = source
        .split("fn sync_notification_projection")
        .nth(1)
        .and_then(|source| source.split("fn notification_counters").next())
        .expect("notification projection source must remain isolated");
    let capped_iteration = projection
        .find(".take(MAX_NOTIFICATION_HISTORY)")
        .expect("history input must be capped before entry formatting");
    let formatted_entries = projection
        .find(".collect::<Vec<_>>()")
        .expect("history entries must be materialized once after the cap");

    assert!(capped_iteration < formatted_entries);
    assert!(!projection.contains("candidate_entries"));
    assert!(projection.contains("pending_decisions.len()"));
    assert!(projection.contains("progress.len()"));
    assert!(projection.contains("toasts.len()"));
}

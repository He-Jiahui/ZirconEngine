#[test]
fn snapshot_releases_framework_locks_before_async_reports() {
    let source = include_str!("../query_environment_runtime_snapshot.rs")
        .split_once("#[cfg(test)]")
        .map(|(production, _)| production)
        .expect("snapshot query production source");
    let finish = source
        .find("framework.finish_submission()?;")
        .expect("pending submission must finish first");
    let operation = source
        .find("framework.lock_operation()")
        .expect("query must serialize renderer access");
    let state = source
        .find("framework.lock_state()")
        .expect("query must take the state lock once");
    let projection = source
        .find("EnvironmentRuntimeSnapshot::try_from_current_reports")
        .expect("query must use the core contract projection");
    let cache_owner = source
        .find("Arc::clone(&state.environment_ibl_hydration_cache)")
        .expect("query must clone the hydration owner under the state lock");
    let cubemap_upload_report = source
        .find("state.renderer.environment_cubemap_upload_report()")
        .expect("query must project cubemap staging while holding renderer state");
    let capture_residency_report = source
        .find("state.environment_capture_residency.resident_gpu_bytes()")
        .expect("query must project capture residency while holding renderer state");
    let capture_residency_epoch = source
        .find("state.environment_capture_residency.observation_epoch()")
        .expect("query must project capture residency observation epoch");
    let state_scope_end = source
        .find("\n    };\n")
        .expect("framework locks must end in an explicit scope");
    let hydration_lock = source
        .find("environment_ibl_hydration_cache\n        .lock()")
        .expect("query must lock hydration after releasing framework state");
    let hydration_report = source
        .find(".report();")
        .expect("query must publish the non-destructive hydration report");
    let capture_report = source
        .find("framework.environment_capture_report()")
        .expect("query must publish the non-destructive capture report");

    assert!(finish < operation && operation < state && state < cubemap_upload_report);
    assert!(cubemap_upload_report < capture_residency_epoch);
    assert!(capture_residency_epoch < capture_residency_report);
    assert!(capture_residency_report < cache_owner);
    assert!(cache_owner < state_scope_end && state_scope_end < hydration_lock);
    assert!(hydration_lock < hydration_report && hydration_report < capture_report);
    assert!(capture_report < projection);
    assert_eq!(source.matches("framework.lock_operation()").count(), 1);
    assert_eq!(source.matches("framework.lock_state()").count(), 1);
    assert!(!source.contains("query_stats("));
    assert!(!source.contains("take_realtime_ibl"));
    assert!(!source.contains("take_completed_gpu_timing_report"));
}

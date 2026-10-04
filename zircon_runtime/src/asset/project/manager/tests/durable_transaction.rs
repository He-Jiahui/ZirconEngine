use crate::core::resource::io::transaction::{DurableCommitReport, DurableRecoveryReport};
use crate::core::runtime::diagnostics::profiling::{
    reset_capture, snapshot, start_capture, test_capture_lock, ProfileCaptureConfig,
};

use super::{record_commit_report, record_recovery_report};

#[test]
fn project_adapter_publishes_resource_neutral_live_rollback_report() {
    let _guard = test_capture_lock();
    let mut config = ProfileCaptureConfig::default();
    config.session_id = "project-durable-live-rollback-report".to_owned();
    config.max_counters = 8;
    start_capture(config);

    record_commit_report(DurableCommitReport::from_activity_counts(3, 2, 1, 1));

    let snapshot = snapshot();
    reset_capture();
    let values = snapshot
        .counters
        .iter()
        .map(|counter| (counter.name.as_str(), counter.value))
        .collect::<std::collections::HashMap<_, _>>();
    assert_eq!(
        values["resource.transaction.live_rollback_restore_attempt_count"],
        3.0
    );
    assert_eq!(
        values["resource.transaction.live_rollback_restore_success_count"],
        2.0
    );
    assert_eq!(
        values["resource.transaction.deferred_commit_recovery_count"],
        1.0
    );
    assert_eq!(values["resource.transaction.deferred_cleanup_count"], 1.0);
}

#[test]
fn project_adapter_publishes_resource_neutral_recovery_report() {
    let _guard = test_capture_lock();
    let mut config = ProfileCaptureConfig::default();
    config.session_id = "project-durable-recovery-report".to_owned();
    config.max_counters = 8;
    start_capture(config);

    record_recovery_report(DurableRecoveryReport::new(2, 3, 4));

    let snapshot = snapshot();
    reset_capture();
    let values = snapshot
        .counters
        .iter()
        .map(|counter| (counter.name.as_str(), counter.value))
        .collect::<std::collections::HashMap<_, _>>();
    assert_eq!(values["resource.transaction.recovery_rollback_count"], 2.0);
    assert_eq!(values["resource.transaction.recovery_cleanup_count"], 3.0);
    assert_eq!(
        values["resource.transaction.intent_orphan_cleanup_count"],
        4.0
    );
}

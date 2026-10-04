use super::{DurableCommitReport, DurableRecoveryReport};

#[test]
fn commit_report_keeps_live_rollback_attempts_separate_from_successes() {
    let mut report = DurableCommitReport::default();
    report.record_rollback_restore_attempt();
    report.record_rollback_restore_attempt();
    report.record_rollback_restore_success();

    assert_eq!(report.rollback_restore_attempt_count(), 2);
    assert_eq!(report.rollback_restore_success_count(), 1);
    assert_eq!(report.deferred_commit_recovery_count(), 0);
    assert_eq!(report.deferred_cleanup_count(), 0);
}

#[test]
fn commit_report_counts_deferred_terminal_cleanup() {
    let mut report = DurableCommitReport::default();
    report.record_deferred_cleanup();

    assert_eq!(report.deferred_cleanup_count(), 1);
}

#[test]
fn commit_report_counts_deferred_commit_recovery() {
    let mut report = DurableCommitReport::default();
    report.record_deferred_commit_recovery();

    assert_eq!(report.deferred_commit_recovery_count(), 1);
}

#[test]
fn recovery_report_keeps_resource_owned_activity_counts_typed() {
    let report = DurableRecoveryReport::new(2, 3, 4);

    assert_eq!(report.rollback_count(), 2);
    assert_eq!(report.cleanup_count(), 3);
    assert_eq!(report.intent_orphan_cleanup_count(), 4);
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DurableCommitReport {
    rollback_restore_attempt_count: usize,
    rollback_restore_success_count: usize,
    deferred_commit_recovery_count: usize,
    deferred_cleanup_count: usize,
}

impl DurableCommitReport {
    pub(super) fn record_rollback_restore_attempt(&mut self) {
        self.rollback_restore_attempt_count = self.rollback_restore_attempt_count.saturating_add(1);
    }

    pub(super) fn record_rollback_restore_success(&mut self) {
        self.rollback_restore_success_count = self.rollback_restore_success_count.saturating_add(1);
    }

    pub(super) fn record_deferred_cleanup(&mut self) {
        self.deferred_cleanup_count = self.deferred_cleanup_count.saturating_add(1);
    }

    pub(super) fn record_deferred_commit_recovery(&mut self) {
        self.deferred_commit_recovery_count = self.deferred_commit_recovery_count.saturating_add(1);
    }

    #[cfg(any(test, feature = "profiling"))]
    pub fn rollback_restore_attempt_count(self) -> usize {
        self.rollback_restore_attempt_count
    }

    #[cfg(any(test, feature = "profiling"))]
    pub fn rollback_restore_success_count(self) -> usize {
        self.rollback_restore_success_count
    }

    #[cfg(any(test, feature = "profiling"))]
    pub fn deferred_cleanup_count(self) -> usize {
        self.deferred_cleanup_count
    }

    #[cfg(any(test, feature = "profiling"))]
    pub fn deferred_commit_recovery_count(self) -> usize {
        self.deferred_commit_recovery_count
    }

    #[cfg(feature = "profiling")]
    pub fn has_commit_activity(self) -> bool {
        self.rollback_restore_attempt_count != 0
            || self.deferred_commit_recovery_count != 0
            || self.deferred_cleanup_count != 0
    }

    #[cfg(any(test, feature = "test-support"))]
    pub fn from_activity_counts(
        rollback_attempts: usize,
        rollback_successes: usize,
        deferred_commit_recoveries: usize,
        deferred_cleanups: usize,
    ) -> Self {
        Self {
            rollback_restore_attempt_count: rollback_attempts,
            rollback_restore_success_count: rollback_successes,
            deferred_commit_recovery_count: deferred_commit_recoveries,
            deferred_cleanup_count: deferred_cleanups,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DurableRecoveryReport {
    rollback_count: usize,
    cleanup_count: usize,
    intent_orphan_cleanup_count: usize,
}

impl DurableRecoveryReport {
    pub fn new(
        rollback_count: usize,
        cleanup_count: usize,
        intent_orphan_cleanup_count: usize,
    ) -> Self {
        Self {
            rollback_count,
            cleanup_count,
            intent_orphan_cleanup_count,
        }
    }

    pub fn rollback_count(self) -> usize {
        self.rollback_count
    }

    pub fn cleanup_count(self) -> usize {
        self.cleanup_count
    }

    pub fn intent_orphan_cleanup_count(self) -> usize {
        self.intent_orphan_cleanup_count
    }
}

#[cfg(test)]
mod tests {
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
}

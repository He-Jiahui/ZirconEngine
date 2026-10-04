use super::{
    ProjectRecoveryTakeoverDisposition, ProjectSessionEffect, ProjectSessionEffectDisposition,
    ProjectSessionEffectLedgerPhase, ProjectSessionEffectRecoveryEntry,
    ProjectSessionRecoveryStatus,
};

#[test]
fn only_a_terminal_ledger_allows_recovery_takeover() {
    assert!(ProjectRecoveryTakeoverDisposition::from_status(
        &ProjectSessionRecoveryStatus::Terminal,
    )
    .allows_takeover());
    assert!(!ProjectRecoveryTakeoverDisposition::from_status(
        &ProjectSessionRecoveryStatus::Missing,
    )
    .allows_takeover());
    assert!(!ProjectRecoveryTakeoverDisposition::from_status(
        &ProjectSessionRecoveryStatus::Incomplete {
            phase: ProjectSessionEffectLedgerPhase::Ready,
            effects: vec![ProjectSessionEffectRecoveryEntry::new(
                ProjectSessionEffect::Runtime,
                ProjectSessionEffectDisposition::Committed,
            )],
        },
    )
    .allows_takeover());
    assert!(!ProjectRecoveryTakeoverDisposition::from_status(
        &ProjectSessionRecoveryStatus::RecoveryRequired {
            phase: ProjectSessionEffectLedgerPhase::RecoveryRequired,
            effects: vec![ProjectSessionEffectRecoveryEntry::new(
                ProjectSessionEffect::Documents,
                ProjectSessionEffectDisposition::RecoveryRequired,
            )],
        },
    )
    .allows_takeover());
}

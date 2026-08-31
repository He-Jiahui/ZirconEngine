use zircon_runtime_interface::project::{
    ProjectActivationOperationId, ProjectActivationOperationIdGenerator, ProjectLaunchInstanceId,
};

use super::{
    ProjectSessionEffect, ProjectSessionEffectDisposition, ProjectSessionEffectLedger,
    ProjectSessionEffectLedgerPhase, ProjectSessionEffectRecoveryEntry,
    ProjectSessionRecoveryStatus,
};

fn operation_id() -> ProjectActivationOperationId {
    ProjectActivationOperationIdGenerator::new(ProjectLaunchInstanceId::new())
        .allocate()
        .expect("fixture operation id")
}

fn commit_activation(ledger: &mut ProjectSessionEffectLedger) {
    for effect in ProjectSessionEffect::ACTIVATION_EFFECTS {
        ledger.prepare(effect).expect("prepare activation effect");
        ledger.commit(effect).expect("commit activation effect");
    }
}

fn commit_close(ledger: &mut ProjectSessionEffectLedger) {
    for effect in ProjectSessionEffect::CLOSE_EFFECTS {
        ledger.commit(effect).expect("commit close effect");
    }
}

#[test]
fn session_effect_ledger_requires_preparation_before_commit() {
    let mut ledger = ProjectSessionEffectLedger::for_operation(operation_id());

    assert!(ledger.commit(ProjectSessionEffect::Runtime).is_err());
    ledger
        .prepare(ProjectSessionEffect::Runtime)
        .expect("runtime effect preparation");
    ledger
        .commit(ProjectSessionEffect::Runtime)
        .expect("prepared runtime effect commit");

    assert_eq!(
        ledger.disposition(ProjectSessionEffect::Runtime),
        Some(ProjectSessionEffectDisposition::Committed)
    );
}

#[test]
fn ready_session_is_recoverable_state_not_terminal_activation_cleanup() {
    let mut ledger = ProjectSessionEffectLedger::for_operation(operation_id());
    commit_activation(&mut ledger);
    ledger.begin_ready().expect("ready phase");

    assert_eq!(ledger.phase(), ProjectSessionEffectLedgerPhase::Ready);
    assert!(matches!(
        ledger.recovery_status(),
        ProjectSessionRecoveryStatus::Incomplete {
            phase: ProjectSessionEffectLedgerPhase::Ready,
            ..
        }
    ));
}

#[test]
fn close_phase_replaces_activation_inventory_and_requires_every_close_owner() {
    let mut ledger = ProjectSessionEffectLedger::for_operation(operation_id());
    commit_activation(&mut ledger);
    ledger.begin_ready().expect("ready phase");
    ledger.begin_closing().expect("closing phase");

    assert_eq!(ledger.phase(), ProjectSessionEffectLedgerPhase::Closing);
    assert_eq!(
        ledger.effects().len(),
        ProjectSessionEffect::CLOSE_EFFECTS.len()
    );
    assert!(ProjectSessionEffect::CLOSE_EFFECTS
        .into_iter()
        .all(|effect| {
            ledger.disposition(effect) == Some(ProjectSessionEffectDisposition::Prepared)
        }));
    assert!(ledger.finish_closed().is_err());

    commit_close(&mut ledger);
    ledger.finish_closed().expect("closed phase");
    assert_eq!(
        ledger.recovery_status(),
        ProjectSessionRecoveryStatus::Terminal
    );
}

#[test]
fn close_failure_preserves_exact_effect_inventory() {
    let mut ledger = ProjectSessionEffectLedger::for_operation(operation_id());
    commit_activation(&mut ledger);
    ledger.begin_ready().expect("ready phase");
    ledger.begin_closing().expect("closing phase");
    ledger
        .mark_recovery_required(ProjectSessionEffect::Runtime)
        .expect("runtime recovery state");

    let ProjectSessionRecoveryStatus::RecoveryRequired { phase, effects } =
        ledger.recovery_status()
    else {
        panic!("close failure must require recovery");
    };
    assert_eq!(phase, ProjectSessionEffectLedgerPhase::RecoveryRequired);
    assert_eq!(effects.len(), ProjectSessionEffect::CLOSE_EFFECTS.len());
    assert_eq!(
        effects
            .iter()
            .find(|entry| entry.effect() == ProjectSessionEffect::Runtime),
        Some(&ProjectSessionEffectRecoveryEntry::new(
            ProjectSessionEffect::Runtime,
            ProjectSessionEffectDisposition::RecoveryRequired,
        ))
    );
    assert!(
        effects
            .iter()
            .filter(|entry| { entry.disposition() == ProjectSessionEffectDisposition::Prepared })
            .count()
            == ProjectSessionEffect::CLOSE_EFFECTS.len() - 1
    );
}

#[test]
fn phase_rejects_effects_from_the_wrong_lifecycle_domain() {
    let mut ledger = ProjectSessionEffectLedger::for_operation(operation_id());

    assert!(ledger.prepare(ProjectSessionEffect::Play).is_err());
    commit_activation(&mut ledger);
    ledger.begin_ready().expect("ready phase");
    assert!(ledger.prepare(ProjectSessionEffect::AssetJobs).is_err());
    ledger.begin_closing().expect("closing phase");
    assert!(ledger
        .prepare(ProjectSessionEffect::RecentProjection)
        .is_err());
}

#[test]
fn every_reachable_persisted_phase_satisfies_the_decode_invariant() {
    let mut activating = ProjectSessionEffectLedger::for_operation(operation_id());
    activating
        .prepare(ProjectSessionEffect::Runtime)
        .expect("activation prepare");
    assert!(activating.validate_persisted_state().is_ok());

    let mut ready = ProjectSessionEffectLedger::for_operation(operation_id());
    commit_activation(&mut ready);
    ready.begin_ready().expect("ready phase");
    ready
        .prepare(ProjectSessionEffect::RecentProjection)
        .expect("recent prepare");
    ready
        .roll_back(ProjectSessionEffect::RecentProjection)
        .expect("recent rollback");
    assert!(ready.validate_persisted_state().is_ok());

    let mut aborted = ProjectSessionEffectLedger::for_operation(operation_id());
    aborted
        .prepare(ProjectSessionEffect::Runtime)
        .expect("aborted runtime prepare");
    aborted
        .roll_back(ProjectSessionEffect::Runtime)
        .expect("aborted runtime rollback");
    aborted
        .finish_aborted_activation()
        .expect("aborted activation terminal phase");
    assert!(aborted.validate_persisted_state().is_ok());

    let mut closing = ProjectSessionEffectLedger::for_operation(operation_id());
    commit_activation(&mut closing);
    closing.begin_ready().expect("ready before close");
    closing.begin_closing().expect("closing phase");
    assert!(closing.validate_persisted_state().is_ok());
    closing
        .mark_recovery_required(ProjectSessionEffect::Runtime)
        .expect("close recovery phase");
    assert!(closing.validate_persisted_state().is_ok());
}

#[test]
fn ready_recent_projection_can_become_an_exact_recovery_owner() {
    let mut ledger = ProjectSessionEffectLedger::for_operation(operation_id());
    commit_activation(&mut ledger);
    ledger.begin_ready().expect("ready phase");
    ledger
        .prepare(ProjectSessionEffect::RecentProjection)
        .expect("recent projection prepare");
    ledger
        .mark_recovery_required(ProjectSessionEffect::RecentProjection)
        .expect("recent projection recovery owner");

    assert_eq!(
        ledger.phase(),
        ProjectSessionEffectLedgerPhase::RecoveryRequired
    );
    assert!(ledger.validate_persisted_state().is_ok());
}

#[test]
fn closing_effects_cannot_roll_back_after_forward_only_teardown_begins() {
    let mut ledger = ProjectSessionEffectLedger::for_operation(operation_id());
    commit_activation(&mut ledger);
    ledger.begin_ready().expect("ready phase");
    ledger.begin_closing().expect("closing phase");

    assert!(ledger.roll_back(ProjectSessionEffect::Runtime).is_err());
    assert_eq!(
        ledger.disposition(ProjectSessionEffect::Runtime),
        Some(ProjectSessionEffectDisposition::Prepared)
    );
    assert!(ledger.validate_persisted_state().is_ok());
}

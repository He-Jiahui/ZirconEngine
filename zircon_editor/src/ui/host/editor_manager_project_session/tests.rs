use std::time::Instant;

use super::super::editor_manager::{
    ProjectSessionHeartbeatState, PROJECT_SESSION_HEARTBEAT_INTERVAL,
};

#[test]
fn project_session_transition_authority_covers_every_manager_lifecycle_entry() {
    let activation_source = include_str!("../editor_manager_project_session.rs");
    let activation = activation_source
        .split("#[cfg(test)]")
        .next()
        .expect("project activation production source");
    let close_source = include_str!("../editor_manager_project.rs");
    let close = close_source
        .split("#[cfg(test)]")
        .next()
        .expect("project close production source");
    let gate = "let _transition = self.begin_project_session_transition()?;";

    assert_eq!(activation.matches(gate).count(), 4);
    assert_eq!(close.matches(gate).count(), 1);

    for (signature, first_operation) in [
        (
            "pub(super) fn open_project_document_with_admission",
            "self.activate_project_from_preflight(",
        ),
        (
            "pub(super) fn open_project_and_remember_with_session",
            "self.activate_project_from_preflight(",
        ),
        (
            "pub(super) fn recover_project_and_remember_with_session",
            "self.activate_project_from_preflight(",
        ),
        (
            "pub(super) fn create_project_and_open_with_session",
            "ProjectAuthority::default()",
        ),
    ] {
        assert_gate_precedes_operation(activation, signature, gate, first_operation);
    }
    assert_gate_precedes_operation(
        close,
        "pub(crate) fn commit_project_close",
        gate,
        "self.ensure_project_recovery_is_settled()?;",
    );
}

fn assert_gate_precedes_operation(
    source: &str,
    signature: &str,
    gate: &str,
    first_operation: &str,
) {
    let function = source
        .split(signature)
        .nth(1)
        .unwrap_or_else(|| panic!("missing lifecycle entry `{signature}`"))
        .split("\n    pub(")
        .next()
        .expect("lifecycle entry body");
    let gate_index = function
        .find(gate)
        .unwrap_or_else(|| panic!("lifecycle entry `{signature}` is not serialized"));
    let operation_index = function
        .find(first_operation)
        .unwrap_or_else(|| panic!("lifecycle entry `{signature}` is missing `{first_operation}`"));

    assert!(
        gate_index < operation_index,
        "lifecycle entry `{signature}` starts work before acquiring the transition authority"
    );
}

#[test]
fn activation_failure_keeps_the_guard_until_rollback_is_terminal() {
    let source = include_str!("../editor_manager_project_session.rs");
    let admission = source
        .split("fn admit_project_session<T>")
        .nth(1)
        .expect("project admission implementation");
    let retained_guard = admission
        .find("Err(activation_failure) if activation_failure.retains_session_guard()")
        .expect("incomplete rollback must retain its exclusive guard");
    let releasable_failure = admission
        .find("Err(activation_failure) => {")
        .expect("complete rollback must release its guard");
    let release_guard = admission[releasable_failure..]
        .find("match guard.release()")
        .expect("releasable activation failures must release their guard");

    assert!(retained_guard < releasable_failure);
    assert!(admission[retained_guard..releasable_failure].contains("*guard_slot = Some(guard);"));
    let releasable = &admission[releasable_failure..];
    assert!(release_guard > 0);
    assert!(releasable.contains("Err(release_error) => {"));
    assert!(releasable.contains("*guard_slot = Some(guard);"));
}

#[test]
fn activation_rollback_cleans_the_terminal_ledger_only_after_guard_release() {
    let source = include_str!("../editor_manager_project_session.rs");
    let releasable = source
        .split("Err(activation_failure) => {")
        .nth(1)
        .expect("releasable activation rollback branch");
    let release = releasable
        .find("match guard.release()")
        .expect("guard release boundary");
    let cleanup = releasable
        .find("ProjectSessionEffectLedgerStore::load")
        .expect("closed-ledger cleanup after release");

    assert!(release < cleanup);
}

#[test]
fn admission_lifecycle_persistence_failures_share_activation_compensation() {
    let source = include_str!("../editor_manager_project_session.rs");
    let admission = source
        .split("fn admit_project_session<T>")
        .nth(1)
        .expect("project admission implementation");
    let compensation_boundary = admission
        .find("let activation = (|| {")
        .expect("all post-claim persistence must enter the activation compensation boundary");
    let activation_match = admission
        .find("match activation {")
        .expect("project admission must resolve the compensated activation result");
    let compensated = &admission[compensation_boundary..activation_match];

    let preflight = compensated
        .find("guard.mark_preflight_approved()")
        .expect("preflight persistence must be compensated after the session claim");
    let activating = compensated
        .find("guard.begin_activation()")
        .expect("activation persistence must be compensated after the session claim");
    let ledger = compensated
        .find("ProjectSessionEffectLedgerStore::create")
        .expect("the durable effect ledger must share the same compensation boundary");

    assert!(preflight < activating);
    assert!(activating < ledger);
    assert!(!admission[..compensation_boundary].contains("guard.mark_preflight_approved()"));
    assert!(!admission[..compensation_boundary].contains("guard.begin_activation()"));
}

#[test]
fn project_materialization_occurs_only_after_the_session_claim() {
    let source = include_str!("../editor_manager_project_session.rs");
    let admission = source
        .split("fn activate_project_from_preflight<T>")
        .nth(1)
        .expect("project activation must have one admission-gated path");
    let claim = admission
        .find("self.admit_project_session")
        .expect("project activation must claim before materialization");
    let materialize = admission
        .find("authority.open_resolved_project")
        .expect("project activation must materialize through ProjectAuthority");

    assert!(
        claim < materialize,
        "ProjectAuthority must not materialize a project before the exclusive admission lease"
    );
    assert!(admission.contains("authority.revalidate_preflight(&preflight)"));
    assert!(admission.contains("current.composition()"));
}

#[test]
fn project_session_transition_recovery_takeover_refreshes_and_matches_the_residual_under_its_owned_lease(
) {
    let source = include_str!("../editor_manager_project_session.rs");
    let recovery_branch = source
        .split("ProjectSessionAdmissionMode::RecoveryTakeover => {")
        .nth(1)
        .expect("recovery takeover admission branch");
    let takeover = recovery_branch
        .find("residual.take_over(admission)")
        .expect("recovery takeover must replace the guarded residual explicitly");
    let gate = &recovery_branch[..takeover];
    let refresh = gate
        .find("ProjectRecoveryAssessment::inspect(project_root)")
        .expect("the restore plan must be refreshed while the residual lease is held");
    let identity = gate
        .find("assessed_residual != residual.record()")
        .expect("the refreshed plan must match the lease-protected residual identity");
    let terminal = gate
        .find("assessment.admission().allows_recovery_takeover()")
        .expect("only a terminal refreshed assessment may take over the residual");

    assert!(refresh < identity);
    assert!(identity < terminal);
    assert!(terminal < takeover);
}

#[test]
fn project_session_heartbeat_is_interval_bounded_and_stops_after_degradation() {
    let start = Instant::now();
    let mut heartbeat = ProjectSessionHeartbeatState::default();

    assert!(!heartbeat.is_due(start));
    heartbeat.activate(start);
    assert!(!heartbeat.is_due(start));
    assert!(!heartbeat.is_due(start + PROJECT_SESSION_HEARTBEAT_INTERVAL / 2));
    assert_eq!(
        heartbeat.next_refresh(),
        Some(start + PROJECT_SESSION_HEARTBEAT_INTERVAL)
    );

    let first_due = start + PROJECT_SESSION_HEARTBEAT_INTERVAL;
    assert!(heartbeat.is_due(first_due));
    heartbeat.mark_refreshed(first_due);
    assert!(!heartbeat.is_due(first_due));
    assert!(!heartbeat.is_due(first_due + PROJECT_SESSION_HEARTBEAT_INTERVAL / 2));

    heartbeat.mark_degraded();
    assert!(!heartbeat.is_due(first_due + PROJECT_SESSION_HEARTBEAT_INTERVAL * 2));
    assert_eq!(heartbeat.next_refresh(), None);

    heartbeat.clear();
    assert!(!heartbeat.is_due(first_due + PROJECT_SESSION_HEARTBEAT_INTERVAL * 3));
}

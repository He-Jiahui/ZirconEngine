use super::super::editor_error::EditorError;
use super::{ProjectActivationFailure, RecentProjectProjectionDisposition};

#[test]
fn failed_recent_project_projection_is_deferred_without_becoming_an_activation_failure() {
    assert_eq!(
        RecentProjectProjectionDisposition::from_result::<(), _>(Err(
            "shared registry is unavailable"
        )),
        RecentProjectProjectionDisposition::Deferred {
            diagnostic: "shared registry is unavailable".to_string(),
        }
    );
}

#[test]
fn incomplete_project_activation_rollback_preserves_published_recovery_state() {
    let complete = ProjectActivationFailure::releasable(EditorError::Project(
        "activation rollback completed".to_string(),
    ));
    let incomplete = ProjectActivationFailure::quarantined(EditorError::Project(
        "runtime project rollback failed".to_string(),
    ));

    assert!(!complete.preserves_published_project_for_recovery());
    assert!(incomplete.preserves_published_project_for_recovery());
}

#[test]
fn activation_effects_cross_the_durable_gate_before_ready() {
    let effects = include_str!("../editor_manager_project_activation_effects.rs");
    let session = include_str!("../editor_manager_project_session.rs");
    let gate_start = effects
        .find("fn run_project_activation_effect<T>(")
        .expect("activation effect gate");
    let gate_end = effects[gate_start..]
        .find("fn complete_project_open(")
        .map(|offset| gate_start + offset)
        .expect("activation effect gate boundary");
    let gate = &effects[gate_start..gate_end];
    let prepare = gate
        .find("ledger.prepare(effect)")
        .expect("effect must be durable-prepared");
    let activate = gate
        .find("activate()")
        .expect("effect must execute after preparation");
    let commit = gate
        .find("ledger.commit(effect)")
        .expect("effect must be durable-committed");
    assert!(prepare < activate && activate < commit);

    for effect in [
        "ProjectSessionEffect::Runtime",
        "ProjectSessionEffect::Diagnostics",
        "ProjectSessionEffect::ProjectPlugins",
        "ProjectSessionEffect::Documents",
        "ProjectSessionEffect::UserInterface",
    ] {
        assert!(
            effects.contains(effect) || session.contains(effect),
            "activation effect `{effect}` must pass through the durable gate"
        );
    }

    let admission_start = session
        .find("fn admit_project_session<T>(")
        .expect("project admission owner");
    let admission = &session[admission_start..];
    let session_prepared = admission
        .find(".prepare(ProjectSessionEffect::Session)")
        .expect("session must be ledger-prepared");
    let ready = admission
        .find("guard.commit_ready()")
        .expect("ready generation commit");
    let session_committed = admission
        .find(".commit(ProjectSessionEffect::Session)")
        .expect("session must be ledger-committed");
    let ledger_ready = admission
        .find("ledger.begin_ready()")
        .expect("ledger must be Ready before publishing the session guard");
    assert!(session_prepared < session_committed);
    assert!(session_committed < ledger_ready && ledger_ready < ready);
}

#[test]
fn recent_project_projection_runs_after_ready_commit_and_outside_project_open_commit_gate() {
    let effects = include_str!("../editor_manager_project_activation_effects.rs");
    let session = include_str!("../editor_manager_project_session.rs");
    let activate_start = session
        .find("fn activate_project_from_preflight<T>(")
        .expect("project activation owner");
    let activate_end = session[activate_start..]
        .find("fn activate_project_from_preflight_with_disposition<T>(")
        .map(|offset| activate_start + offset)
        .expect("preflight-project activation boundary");
    let activate = &session[activate_start..activate_end];
    let admission_call = activate
        .find(".activate_project_from_preflight_with_disposition(")
        .expect("project activation must pass through the admission owner");
    let recent_projection = activate
        .find("self.finalize_project_activation(completion)")
        .expect("recent projection must run after the committed activation result");
    let disposition_end = session[activate_end..]
        .find("fn activate_prepared_project_after_admission<T>(")
        .map(|offset| activate_end + offset)
        .expect("admission owner boundary");
    assert!(session[activate_end..disposition_end].contains(".admit_project_session("));
    let admission_start = session
        .find("fn admit_project_session<T>(")
        .expect("project admission owner");
    let admission = &session[admission_start..];
    assert!(admission_call < recent_projection);
    assert!(
        admission.contains("guard.commit_ready()"),
        "admission must commit Ready before its caller may finalize projections"
    );
    let complete_open_start = effects
        .find("fn complete_project_open(")
        .expect("project-open completion owner");
    let complete_open_end = effects[complete_open_start..]
        .find("fn finalize_project_activation")
        .map(|offset| complete_open_start + offset)
        .expect("post-commit projection boundary");
    assert!(
        !effects[complete_open_start..complete_open_end].contains("record_recent_project("),
        "recent history must not participate in the project-open commit gate"
    );
    assert!(
        effects.contains("fn finalize_project_activation")
            && effects.contains("ProjectSessionEffect::RecentProjection"),
        "recent history is a separately tracked post-Ready projection"
    );
}

#[test]
fn project_activation_consumes_only_preflight_approved_plugin_capabilities() {
    let source = include_str!("../editor_manager_project_activation_effects.rs");
    let complete_start = source
        .find("fn complete_project_open(")
        .expect("project-open completion owner");
    let complete_end = source[complete_start..]
        .find("fn finalize_project_activation")
        .map(|offset| complete_start + offset)
        .expect("post-open projection boundary");
    let complete = &source[complete_start..complete_end];

    assert!(complete.contains("composition.approved_project_plugins()"));
    assert!(complete.contains("composition.allows_native_extensions()"));
    assert!(!complete.contains("&document.manifest),"));
}

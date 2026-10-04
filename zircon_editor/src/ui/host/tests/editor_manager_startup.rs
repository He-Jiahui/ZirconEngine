#[test]
fn recovery_profile_defers_authoritative_assessment_to_leased_admission() {
    let source = include_str!("../editor_manager_startup.rs");
    let production = source
        .split("#[cfg(test)]")
        .next()
        .expect("startup production source");
    let recovery_profile = production
        .find("ProjectLaunchProfile::Recovery")
        .expect("recovery launch branch");
    let takeover = production
        .find("self.recover_project_and_remember_with_session")
        .expect("recovery takeover dispatch");

    assert!(recovery_profile < takeover);
    assert!(!production.contains("require_recovery_profile_takeover"));
}

#[test]
fn project_session_transition_recovery_decisions_remain_serialized_and_fail_closed() {
    let session = include_str!("../editor_manager_project_session.rs");
    let recovery = session
        .split("pub(super) fn recover_project_and_remember_with_session")
        .nth(1)
        .expect("serialized recovery activation implementation");
    let gate = recovery
        .find("self.begin_project_session_transition()?")
        .expect("recovery activation must hold the transition gate");
    let activate = recovery
        .find("self.activate_project_from_preflight(")
        .expect("recovery activation call");
    let begin = recovery
        .find("self.begin_project_recovery_decisions(")
        .expect("recovery decisions must begin before releasing the transition gate");
    let retain = recovery
        .find("self.retain_project_session_for_recovery(error)")
        .expect("recovery coordinator failure must retain the exclusive recovery fence");

    assert!(gate < activate);
    assert!(activate < begin);
    assert!(begin < retain);
}

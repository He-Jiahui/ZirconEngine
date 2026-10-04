use super::*;

#[test]
fn backend_contract_projects_to_stable_neutral_diagnostics() {
    let projected = project_backend_status(PhysicsBackendStatus {
        requested_backend: "jolt".to_string(),
        active_backend: Some("jolt".to_string()),
        state: PhysicsBackendState::Ready,
        detail: Some("native backend active".to_string()),
        simulation_mode: PhysicsSimulationMode::QueryOnly,
        feature_gate: Some("backend-jolt".to_string()),
    });

    assert_eq!(projected.requested_backend, "jolt");
    assert_eq!(projected.active_backend.as_deref(), Some("jolt"));
    assert_eq!(projected.state, "ready");
    assert_eq!(projected.detail.as_deref(), Some("native backend active"));
    assert_eq!(projected.simulation_mode, "query_only");
    assert_eq!(projected.feature_gate.as_deref(), Some("backend-jolt"));
}

#[test]
fn backend_contract_enum_names_are_complete_and_stable() {
    assert_eq!(
        backend_state_name(PhysicsBackendState::Disabled),
        "disabled"
    );
    assert_eq!(
        backend_state_name(PhysicsBackendState::Unavailable),
        "unavailable"
    );
    assert_eq!(backend_state_name(PhysicsBackendState::Ready), "ready");

    assert_eq!(
        simulation_mode_name(PhysicsSimulationMode::Disabled),
        "disabled"
    );
    assert_eq!(
        simulation_mode_name(PhysicsSimulationMode::Simulate),
        "simulate"
    );
    assert_eq!(
        simulation_mode_name(PhysicsSimulationMode::QueryOnly),
        "query_only"
    );
}

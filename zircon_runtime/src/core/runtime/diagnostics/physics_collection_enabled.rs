use crate::core::framework::physics::{
    PhysicsBackendState, PhysicsBackendStatus, PhysicsSimulationMode,
};
use crate::core::manager::resolve_physics_manager;
use crate::core::CoreHandle;

use super::{RuntimePhysicsBackendDiagnostics, RuntimePhysicsDiagnostics};

pub(super) fn collect(core: &CoreHandle) -> RuntimePhysicsDiagnostics {
    let physics = match resolve_physics_manager(core) {
        Ok(physics) => physics,
        Err(error) => return RuntimePhysicsDiagnostics::unavailable(error.to_string()),
    };
    let settings = physics.settings();

    RuntimePhysicsDiagnostics {
        available: true,
        backend_name: Some(physics.backend_name()),
        backend_status: Some(project_backend_status(physics.backend_status())),
        fixed_hz: Some(settings.fixed_hz),
        error: None,
    }
}

fn project_backend_status(status: PhysicsBackendStatus) -> RuntimePhysicsBackendDiagnostics {
    RuntimePhysicsBackendDiagnostics {
        requested_backend: status.requested_backend,
        active_backend: status.active_backend,
        state: backend_state_name(status.state).to_string(),
        detail: status.detail,
        simulation_mode: simulation_mode_name(status.simulation_mode).to_string(),
        feature_gate: status.feature_gate,
    }
}

fn backend_state_name(state: PhysicsBackendState) -> &'static str {
    match state {
        PhysicsBackendState::Disabled => "disabled",
        PhysicsBackendState::Unavailable => "unavailable",
        PhysicsBackendState::Ready => "ready",
    }
}

fn simulation_mode_name(mode: PhysicsSimulationMode) -> &'static str {
    match mode {
        PhysicsSimulationMode::Disabled => "disabled",
        PhysicsSimulationMode::Simulate => "simulate",
        PhysicsSimulationMode::QueryOnly => "query_only",
    }
}

#[cfg(test)]
mod tests {
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
}

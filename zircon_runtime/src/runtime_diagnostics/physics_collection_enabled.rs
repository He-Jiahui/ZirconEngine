//! 启用 physics-contracts 时从 manager service 读取后端状态，投影为稳定的诊断字符串和可选字段。
use crate::core::diagnostics::{RuntimePhysicsBackendDiagnostics, RuntimePhysicsDiagnostics};
use crate::core::framework::physics::{
    PhysicsBackendState, PhysicsBackendStatus, PhysicsSimulationMode,
};
use crate::core::manager::{physics_manager_handle, resolve_manager_service};
use crate::core::CoreHandle;

// registry 解析失败直接转为 unavailable；成功后只读取 settings/backend_status，不在诊断采集路径推进物理状态。
pub(super) fn collect(core: &CoreHandle) -> RuntimePhysicsDiagnostics {
    let physics = match physics_manager_handle(core)
        .and_then(|handle| resolve_manager_service(core, handle))
    {
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
#[path = "tests/physics_collection_enabled.rs"]
mod tests;

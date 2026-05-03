use zircon_runtime::core::framework::physics::{
    PhysicsBackendState, PhysicsBackendStatus, PhysicsSettings, PhysicsSimulationMode,
};

pub const JOLT_ENABLED: bool = cfg!(feature = "jolt");

const BUILTIN_BACKEND_NAME: &str = "builtin";
const JOLT_BACKEND_NAME: &str = "jolt";
const UNCONFIGURED_BACKEND_NAME: &str = "unconfigured";
const JOLT_BACKEND_AVAILABLE: bool = false;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PhysicsRuntimeBackend {
    Disabled,
    Builtin,
    Unavailable,
}

impl PhysicsRuntimeBackend {
    pub(crate) fn allows_sync(self) -> bool {
        matches!(self, Self::Builtin)
    }

    pub(crate) fn allows_step(self, simulation_mode: PhysicsSimulationMode) -> bool {
        matches!(self, Self::Builtin) && simulation_mode == PhysicsSimulationMode::Simulate
    }
}

pub(crate) fn select_runtime_backend(settings: &PhysicsSettings) -> PhysicsRuntimeBackend {
    if settings.simulation_mode == PhysicsSimulationMode::Disabled {
        return PhysicsRuntimeBackend::Disabled;
    }

    if settings.backend.eq_ignore_ascii_case(BUILTIN_BACKEND_NAME) {
        return PhysicsRuntimeBackend::Builtin;
    }

    PhysicsRuntimeBackend::Unavailable
}

pub(crate) fn default_backend_name() -> String {
    if JOLT_BACKEND_AVAILABLE {
        JOLT_BACKEND_NAME.to_string()
    } else {
        UNCONFIGURED_BACKEND_NAME.to_string()
    }
}

pub(crate) fn default_simulation_mode() -> PhysicsSimulationMode {
    if JOLT_BACKEND_AVAILABLE {
        PhysicsSimulationMode::Simulate
    } else {
        PhysicsSimulationMode::Disabled
    }
}

pub(crate) fn physics_backend_status(settings: &PhysicsSettings) -> PhysicsBackendStatus {
    let requested_backend = settings.backend.clone();
    let feature_gate = requested_backend
        .eq_ignore_ascii_case(JOLT_BACKEND_NAME)
        .then_some(JOLT_BACKEND_NAME.to_string());

    match select_runtime_backend(settings) {
        PhysicsRuntimeBackend::Disabled => PhysicsBackendStatus {
            requested_backend,
            active_backend: None,
            state: PhysicsBackendState::Disabled,
            detail: Some("physics simulation is disabled".to_string()),
            simulation_mode: settings.simulation_mode,
            feature_gate,
        },
        PhysicsRuntimeBackend::Builtin => PhysicsBackendStatus {
            active_backend: Some(BUILTIN_BACKEND_NAME.to_string()),
            requested_backend,
            state: PhysicsBackendState::Ready,
            detail: None,
            simulation_mode: settings.simulation_mode,
            feature_gate,
        },
        PhysicsRuntimeBackend::Unavailable => PhysicsBackendStatus {
            detail: Some(unavailable_backend_detail(settings)),
            requested_backend,
            active_backend: None,
            state: PhysicsBackendState::Unavailable,
            simulation_mode: settings.simulation_mode,
            feature_gate,
        },
    }
}

fn unavailable_backend_detail(settings: &PhysicsSettings) -> String {
    if settings.backend.eq_ignore_ascii_case(JOLT_BACKEND_NAME) {
        if JOLT_ENABLED {
            "feature `jolt` is enabled, but no runtime Jolt backend is linked".to_string()
        } else {
            "feature `jolt` is not enabled; physics runs in downgrade mode".to_string()
        }
    } else if settings.backend.trim().is_empty()
        || settings
            .backend
            .eq_ignore_ascii_case(UNCONFIGURED_BACKEND_NAME)
    {
        "no physics backend is configured".to_string()
    } else {
        format!("physics backend `{}` is not available", settings.backend)
    }
}

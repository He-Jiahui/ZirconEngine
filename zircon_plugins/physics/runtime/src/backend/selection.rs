//! 根据 PhysicsSettings 和编译 feature 选择 runtime provider，并把选择结果投影为对外状态。

use zircon_runtime::core::framework::physics::{
    PhysicsBackendState, PhysicsBackendStatus, PhysicsSettings, PhysicsSimulationMode,
};

/// 仅表示本 crate 是否编译了 Jolt feature，不表示原生 world 已创建或通过运行时资格检查。
pub const JOLT_ENABLED: bool = cfg!(feature = "backend-jolt");

const BUILTIN_BACKEND_NAME: &str = "builtin";
const JOLT_BACKEND_NAME: &str = "jolt";
const UNCONFIGURED_BACKEND_NAME: &str = "unconfigured";
const JOLT_BACKEND_AVAILABLE: bool = cfg!(feature = "backend-jolt");

/// manager 用于分派当前 world 的 provider 状态。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PhysicsRuntimeBackend {
    Disabled,
    Builtin,
    Jolt,
    Unavailable,
}

impl PhysicsRuntimeBackend {
    pub(crate) fn allows_step(self, simulation_mode: PhysicsSimulationMode) -> bool {
        matches!(self, Self::Builtin | Self::Jolt)
            && simulation_mode == PhysicsSimulationMode::Simulate
    }
}

// 选择基于 settings 名称与 feature gate；这里不会创建 backend 或 native world。
pub(crate) fn select_runtime_backend(settings: &PhysicsSettings) -> PhysicsRuntimeBackend {
    if settings.simulation_mode == PhysicsSimulationMode::Disabled {
        return PhysicsRuntimeBackend::Disabled;
    }

    if settings.backend.eq_ignore_ascii_case(BUILTIN_BACKEND_NAME) {
        return PhysicsRuntimeBackend::Builtin;
    }

    if JOLT_BACKEND_AVAILABLE && settings.backend.eq_ignore_ascii_case(JOLT_BACKEND_NAME) {
        return PhysicsRuntimeBackend::Jolt;
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

// TODO: [CR-PHYSICS-BACKEND-0008] 这里按 settings/feature 把 Jolt 选为 Ready；首次同步前且 manager 尚无已记录错误时，该状态表示可选/配置可用，并不证明 native world 已创建或通过资格检查。
// 需确认 Ready 的阶段语义；manager/service.rs::backend_status 会在 last_backend_error 存在时覆写为 Unavailable。证据：manager/jolt_world.rs::synchronize_jolt_world、manager/service.rs::backend_status；关联 PH-P1-003。
pub(crate) fn physics_backend_status(settings: &PhysicsSettings) -> PhysicsBackendStatus {
    let requested_backend = settings.backend.clone();
    let feature_gate = requested_backend
        .eq_ignore_ascii_case(JOLT_BACKEND_NAME)
        .then_some("backend-jolt".to_string());

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
        PhysicsRuntimeBackend::Jolt => PhysicsBackendStatus {
            active_backend: Some(JOLT_BACKEND_NAME.to_string()),
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
            "feature `backend-jolt` is enabled, but no runtime Jolt backend is linked".to_string()
        } else {
            "feature `backend-jolt` is not enabled; physics runs in downgrade mode".to_string()
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

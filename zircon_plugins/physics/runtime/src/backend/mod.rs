//! 组织物理后端私有实现与 crate 公共 API：按 feature 纳入 Jolt，重导出稳定接口，其余解析和选择 helper 留在 crate 内。

pub(crate) mod builtin;

#[cfg(feature = "backend-jolt")]
mod jolt;

mod contract;
mod error;
mod handle_pool;
mod handles;
mod mass_properties;
mod selection;
mod types;
mod validation;

pub use builtin::BuiltinPhysicsBackend;
pub use contract::PhysicsBackend;
pub use error::{PhysicsBackendError, PhysicsBackendObjectKind};
pub use handles::{BodyHandle, ConstraintHandle, ShapeHandle};
#[cfg(feature = "backend-jolt")]
pub use jolt::JoltPhysicsBackend;
pub use selection::JOLT_ENABLED;
pub use types::{BodyCommand, BodyDesc, ConstraintDesc, PhysicsEventBuffer};

pub(crate) use mass_properties::resolve_body_mass;

pub(crate) use selection::{
    default_backend_name, default_simulation_mode, physics_backend_status, select_runtime_backend,
    PhysicsRuntimeBackend,
};

#[cfg(test)]
#[path = "tests/cases.rs"]
mod tests;

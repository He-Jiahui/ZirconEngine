//! Physics implementation types consumed by `zircon_runtime::extensions::physics`.

pub const JOLT_ENABLED: bool = cfg!(feature = "jolt");

#[derive(Clone, Debug, Default)]
pub struct PhysicsDriver;

#[derive(Clone, Debug, Default)]
pub struct PhysicsManager;

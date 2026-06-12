//! Neutral contracts for plugin-to-plugin interface calls.

use serde::{Deserialize, Serialize};

/// Marker trait for typed plugin interfaces exported through the runtime bridge table.
///
/// Implementations must be `Send + Sync + 'static` and must not borrow from an ECS `World`.
/// World access belongs in scheduled systems so the scheduler can keep conflict information
/// authoritative.
pub trait PluginInterface: Send + Sync + 'static {
    /// Globally unique interface id such as `physics.query.v1`.
    const INTERFACE_ID: &'static str;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum BridgeError {
    /// The target interface is installed but disabled or currently between generations.
    NotEnabled,
    /// The target interface is not installed in the frozen bridge table.
    Absent,
}

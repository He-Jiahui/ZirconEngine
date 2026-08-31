use serde::{Deserialize, Serialize};

use super::InterfaceSlot;

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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BridgeInterfaceStatus {
    Absent,
    Enabled,
    Disabled,
}

impl BridgeInterfaceStatus {
    pub(crate) fn from_installed_entry(generation: u32, provider_installed: bool) -> Self {
        if generation % 2 == 0 && provider_installed {
            Self::Enabled
        } else {
            Self::Disabled
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BridgeOwnerTransitionMode {
    Activate,
    Disable,
    Deactivate,
    Reload,
}

/// Neutral read/call surface consumed by runtime domains that invoke bridge methods.
///
/// Concrete plugin registration and lifecycle ownership stays in the plugin domain. Consumers
/// depend only on this frozen-table view, so they do not need the plugin facade or manifest types.
pub trait BridgeInvocationTable: Clone + Send + Sync + 'static {
    fn resolve_interface_slot(&self, interface_id: &str) -> Option<InterfaceSlot>;

    fn interface_status_at(&self, slot: InterfaceSlot) -> BridgeInterfaceStatus;

    fn record_enabled_call(&self, slot: InterfaceSlot);

    fn record_not_enabled_call(&self, slot: InterfaceSlot);
}

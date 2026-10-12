//! Neutral contracts for plugin-to-plugin interface calls.

#[path = "contract.rs"]
mod contract;
#[path = "diagnostics.rs"]
mod diagnostics;
#[path = "interface_slot.rs"]
mod interface_slot;
#[path = "strong.rs"]
mod strong;

pub use contract::{
    BridgeError, BridgeInterfaceStatus, BridgeInvocationTable, BridgeOwnerTransitionMode,
    PluginInterface,
};
pub(crate) use diagnostics::BridgeDiagnostics;
pub use diagnostics::BridgeDiagnosticsSnapshot;
pub use interface_slot::InterfaceSlot;
pub use strong::StrongBridge;

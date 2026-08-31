//! Neutral contracts for plugin-to-plugin interface calls.

mod contract;
mod diagnostics;
mod interface_slot;
mod strong;

pub use contract::{
    BridgeError, BridgeInterfaceStatus, BridgeInvocationTable, BridgeOwnerTransitionMode,
    PluginInterface,
};
pub(crate) use diagnostics::BridgeDiagnostics;
pub use diagnostics::BridgeDiagnosticsSnapshot;
pub use interface_slot::InterfaceSlot;
pub use strong::StrongBridge;

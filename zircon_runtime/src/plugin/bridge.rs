mod diagnostics;
mod interface_id;
mod strong;
mod table;
mod weak;

pub use diagnostics::BridgeDiagnosticsSnapshot;
pub use interface_id::InterfaceSlot;
pub use strong::StrongBridge;
pub use table::{
    BridgeEntry, BridgeInterfaceSnapshot, BridgeInterfaceStatus, BridgeOwnerTransitionMode,
    BridgeOwnerTransitionReport, FrozenBridgeTable, InterfaceExport,
};
pub use weak::{BridgeGuard, WeakBridge};

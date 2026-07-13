mod table;
mod weak;

pub use table::{
    BridgeDiagnosticsMatrix, BridgeEntry, BridgeInterfaceSnapshot, BridgeOwnerTransitionReport,
    BridgeTableDiagnosticsSummary, FrozenBridgeTable, InterfaceExport,
};
pub use weak::{BridgeGuard, WeakBridge};

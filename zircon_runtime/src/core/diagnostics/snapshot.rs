use super::{
    DiagnosticStoreSnapshot, RuntimeAnimationDiagnostics, RuntimePhysicsDiagnostics,
    RuntimeRenderDiagnostics,
};
use zircon_runtime_interface::ProfileSnapshot;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct RuntimeDiagnosticsSnapshot {
    pub render: RuntimeRenderDiagnostics,
    pub physics: RuntimePhysicsDiagnostics,
    pub animation: RuntimeAnimationDiagnostics,
    pub store: DiagnosticStoreSnapshot,
    pub profile: ProfileSnapshot,
}

use super::{RuntimeAnimationDiagnostics, RuntimePhysicsDiagnostics, RuntimeRenderDiagnostics};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct RuntimeDiagnosticsSnapshot {
    pub render: RuntimeRenderDiagnostics,
    pub physics: RuntimePhysicsDiagnostics,
    pub animation: RuntimeAnimationDiagnostics,
}

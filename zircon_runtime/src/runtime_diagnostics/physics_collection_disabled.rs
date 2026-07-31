use crate::core::diagnostics::RuntimePhysicsDiagnostics;
use crate::core::CoreHandle;

pub(super) fn collect(_core: &CoreHandle) -> RuntimePhysicsDiagnostics {
    RuntimePhysicsDiagnostics::unavailable("physics contracts are not compiled")
}

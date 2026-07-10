use crate::core::CoreHandle;

use super::RuntimePhysicsDiagnostics;

pub(super) fn collect(_core: &CoreHandle) -> RuntimePhysicsDiagnostics {
    RuntimePhysicsDiagnostics::unavailable("physics contracts are not compiled")
}

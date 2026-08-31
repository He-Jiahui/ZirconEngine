//! Facade-owned runtime diagnostics collectors that resolve registered manager services.

mod collect;
#[cfg(feature = "physics-contracts")]
#[path = "physics_collection_enabled.rs"]
mod physics_collection;
#[cfg(not(feature = "physics-contracts"))]
#[path = "physics_collection_disabled.rs"]
mod physics_collection;

use crate::core::diagnostics::{
    project_runtime_devtools_snapshot, RuntimeDevtoolsSnapshot, RuntimeDiagnosticsSnapshot,
};
use crate::core::CoreHandle;

pub use collect::collect_runtime_diagnostics;
pub(crate) use collect::{
    collect_runtime_diagnostic_current_store, collect_runtime_diagnostic_store,
};

impl CoreHandle {
    pub fn runtime_diagnostics_snapshot(&self) -> RuntimeDiagnosticsSnapshot {
        collect_runtime_diagnostics(self)
    }
}

pub fn collect_runtime_devtools_snapshot(core: &CoreHandle) -> RuntimeDevtoolsSnapshot {
    let diagnostics = collect_runtime_diagnostics(core);
    project_runtime_devtools_snapshot(core, &diagnostics)
}

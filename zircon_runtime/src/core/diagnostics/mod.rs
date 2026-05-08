//! Read-only runtime diagnostics snapshots for editor and tooling surfaces.

mod animation;
mod collect;
mod devtools;
mod physics;
mod render;
mod snapshot;
mod store;

pub use animation::RuntimeAnimationDiagnostics;
pub use collect::collect_runtime_diagnostics;
pub use devtools::{
    collect_runtime_devtools_snapshot, RuntimeDevtoolsBackendStatus,
    RuntimeDevtoolsDiagnosticsSummary, RuntimeDevtoolsModuleSnapshot,
    RuntimeDevtoolsPluginCatalogEntry, RuntimeDevtoolsSceneHookSnapshot,
    RuntimeDevtoolsServiceSnapshot, RuntimeDevtoolsSnapshot,
};
pub use physics::RuntimePhysicsDiagnostics;
pub use render::RuntimeRenderDiagnostics;
pub use snapshot::RuntimeDiagnosticsSnapshot;
pub use store::{
    DiagnosticMeasurement, DiagnosticPath, DiagnosticSeriesSnapshot, DiagnosticStore,
    DiagnosticStoreSnapshot,
};

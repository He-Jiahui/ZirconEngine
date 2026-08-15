//! Read-only runtime diagnostics snapshots for editor and tooling surfaces.

mod animation;
mod devtools;
mod frame_diagnostics;
mod physics;
mod physics_backend;
pub mod profiling;
mod render;
mod render_stats_store;
mod snapshot;
mod store;

pub use animation::RuntimeAnimationDiagnostics;
pub(crate) use devtools::project_runtime_devtools_snapshot;
pub use devtools::{
    RuntimeDevtoolsBackendStatus, RuntimeDevtoolsDiagnosticsSummary, RuntimeDevtoolsModuleSnapshot,
    RuntimeDevtoolsPluginCatalogEntry, RuntimeDevtoolsServiceSnapshot, RuntimeDevtoolsSnapshot,
};
pub use frame_diagnostics::{FrameDiagnostics, FrameDiagnosticsStatus};
pub use physics::RuntimePhysicsDiagnostics;
pub use physics_backend::RuntimePhysicsBackendDiagnostics;
pub use profiling::{
    analyze_counter_hotspots, analyze_hotspots, feature_enabled as profiling_feature_enabled,
    start_capture, stop_capture, CounterHotspotEntry, CounterHotspotReport, HotspotReport,
    ProfileCaptureConfig, ProfileCounterSnapshot, ProfileExportError, ProfileExportReport,
    ProfileExportResult, ProfileFrameScope, ProfileFrameSnapshot, ProfileRecorder,
    ProfileRecorderStatus, ProfileScope, ProfileSnapshot, ProfileSpanSnapshot,
};
pub use render::RuntimeRenderDiagnostics;
pub(crate) use render_stats_store::record_render_stats_diagnostics;
pub use snapshot::RuntimeDiagnosticsSnapshot;
pub use store::{
    DiagnosticMeasurement, DiagnosticPath, DiagnosticSeriesSnapshot, DiagnosticStore,
    DiagnosticStoreSnapshot,
};

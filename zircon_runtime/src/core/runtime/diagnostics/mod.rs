//! Read-only runtime diagnostics snapshots for editor and tooling surfaces.

mod animation;
mod collect;
mod devtools;
mod frame_diagnostics;
mod physics;
pub mod profiling;
mod render;
mod render_stats_store;
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
pub use frame_diagnostics::{FrameDiagnostics, FrameDiagnosticsStatus};
pub use physics::RuntimePhysicsDiagnostics;
pub use profiling::{
    analyze_counter_hotspots, analyze_hotspots, feature_enabled as profiling_feature_enabled,
    start_capture, stop_capture, CounterHotspotEntry, CounterHotspotReport, HotspotReport,
    ProfileCaptureConfig, ProfileCounterSnapshot, ProfileExportReport, ProfileFrameScope,
    ProfileFrameSnapshot, ProfileRecorder, ProfileRecorderStatus, ProfileScope, ProfileSnapshot,
    ProfileSpanSnapshot,
};
pub use render::RuntimeRenderDiagnostics;
pub use snapshot::RuntimeDiagnosticsSnapshot;
pub use store::{
    DiagnosticMeasurement, DiagnosticPath, DiagnosticSeriesSnapshot, DiagnosticStore,
    DiagnosticStoreSnapshot,
};

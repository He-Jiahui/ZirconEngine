//! Read-only runtime diagnostics snapshots for editor and tooling surfaces.
//! 这里定义跨模块只读数据契约；采集器在 runtime_diagnostics 中解析服务，再供编辑器和日志使用。

#[path = "animation.rs"]
mod animation;
#[path = "devtools.rs"]
mod devtools;
#[path = "frame_diagnostics.rs"]
mod frame_diagnostics;
#[path = "physics.rs"]
mod physics;
#[path = "physics_backend.rs"]
mod physics_backend;
#[path = "profiling/mod.rs"]
pub mod profiling;
#[path = "render.rs"]
mod render;
#[path = "render_stats_store/mod.rs"]
mod render_stats_store;
#[path = "snapshot.rs"]
mod snapshot;
#[path = "store.rs"]
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
    DiagnosticMeasurement, DiagnosticPath, DiagnosticSeriesCurrentSnapshot,
    DiagnosticSeriesSnapshot, DiagnosticStore, DiagnosticStoreCurrentSnapshot,
    DiagnosticStoreSnapshot,
};

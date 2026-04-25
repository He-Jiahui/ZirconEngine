//! Read-only runtime diagnostics snapshots for editor and tooling surfaces.

mod animation;
mod collect;
mod physics;
mod render;
mod snapshot;

pub use animation::RuntimeAnimationDiagnostics;
pub use collect::collect_runtime_diagnostics;
pub use physics::RuntimePhysicsDiagnostics;
pub use render::RuntimeRenderDiagnostics;
pub use snapshot::RuntimeDiagnosticsSnapshot;

//! Runtime world inspection snapshots built from ECS hierarchy and reflection state.

mod artifact;
mod hierarchy;
mod snapshot;
mod subscription;

#[cfg(test)]
mod tests;

pub(super) use artifact::WorldInspectionArtifactCache;
pub use artifact::{
    WorldInspectionArtifact, WorldInspectionArtifactDiagnostics, WorldInspectionDelta,
    WorldInspectionFieldDelta, WorldInspectionFieldPath, WorldInspectionFieldsArtifact,
    WorldInspectionSummary,
};
pub use hierarchy::WorldInspectionHierarchyRow;
pub use subscription::{SubscriptionTable, SubscriptionTableDiagnostics, SubscriptionTableLimits};
pub use zircon_runtime_interface::world_sync::WorldInspectionFieldRow as WorldInspectionField;

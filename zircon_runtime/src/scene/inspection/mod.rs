//! Runtime world inspection snapshots built from ECS hierarchy and reflection state.

#[path = "artifact/mod.rs"]
mod artifact;
#[path = "hierarchy.rs"]
mod hierarchy;
#[path = "snapshot.rs"]
mod snapshot;
#[path = "subscription.rs"]
mod subscription;

#[cfg(test)]
#[path = "tests/cases.rs"]
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

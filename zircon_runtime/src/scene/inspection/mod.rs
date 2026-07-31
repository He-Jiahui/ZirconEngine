//! Runtime world inspection snapshots built from ECS hierarchy and reflection state.

mod artifact;
mod field;
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
pub use field::WorldInspectionField;
pub use hierarchy::WorldInspectionHierarchyRow;
pub use snapshot::WorldInspection;
pub use subscription::{SubscriptionTable, SubscriptionTableDiagnostics, SubscriptionTableLimits};

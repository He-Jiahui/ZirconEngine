//! Runtime world inspection snapshots built from ECS hierarchy and reflection state.

mod field;
mod hierarchy;
mod snapshot;

pub use field::WorldInspectionField;
pub use hierarchy::WorldInspectionHierarchyRow;
pub use snapshot::WorldInspection;

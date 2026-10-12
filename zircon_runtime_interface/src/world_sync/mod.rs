//! Transport-neutral world query, watch, and invalidation contracts.

#[path = "invalidation.rs"]
mod invalidation;
#[path = "lifecycle_target_change.rs"]
mod lifecycle_target_change;
#[path = "query.rs"]
mod query;
#[path = "watch.rs"]
mod watch;

pub use invalidation::{AssetReloadFrameApplyReportDto, InvalidationBatch, WorldFact};
pub use lifecycle_target_change::LifecycleTargetChange;
pub use query::{
    ComponentSelector, ComponentWorldQuery, EntityId, EntityRow, QueryFilter, WorldHierarchyQuery,
    WorldHierarchyRow, WorldInspectionFieldRow, WorldInspectionFieldsQuery, WorldQuery,
    WorldQueryResult, WorldTransformSnapshotQuery,
};
pub use watch::{WatchKey, WatchRegistration, WatchToken};

//! Transport-neutral world query, watch, and invalidation contracts.

mod invalidation;
mod query;
mod watch;

pub use invalidation::{AssetReloadFrameApplyReportDto, InvalidationBatch, WorldFact};
pub use query::{
    ComponentSelector, ComponentWorldQuery, EntityId, EntityRow, QueryFilter, WorldHierarchyQuery,
    WorldHierarchyRow, WorldInspectionFieldRow, WorldInspectionFieldsQuery, WorldQuery,
    WorldQueryResult, WorldTransformSnapshotQuery,
};
pub use watch::{WatchKey, WatchRegistration, WatchToken};

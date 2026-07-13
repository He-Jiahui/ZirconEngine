//! Transport-neutral world query, watch, and invalidation contracts.

mod invalidation;
mod query;
mod watch;

pub use invalidation::{AssetReloadFrameApplyReportDto, InvalidationBatch, WorldFact};
pub use query::{
    ComponentSelector, EntityId, EntityRow, QueryFilter, WorldQuery, WorldQueryResult,
};
pub use watch::{WatchKey, WatchRegistration, WatchToken};

//! Editor-owned projection state for transport-neutral runtime synchronization.

mod watch_map;

pub use watch_map::{WorldWatchBinding, WorldWatchMap, WorldWatchMapError, WorldWatchProjection};

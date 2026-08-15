//! Editor-owned projection state for transport-neutral runtime synchronization.

mod pump;
mod watch_map;

pub use pump::{WorldSyncPump, WorldSyncPumpError, WorldSyncPumpReport, TOPIC_WORLD_FACT};
pub use watch_map::{WorldWatchBinding, WorldWatchMap, WorldWatchMapError, WorldWatchProjection};

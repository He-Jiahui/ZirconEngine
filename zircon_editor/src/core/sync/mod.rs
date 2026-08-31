//! Editor-owned projection state for transport-neutral runtime synchronization.

mod pump;
mod watch_map;

pub use pump::{
    QualifiedWatchToken, WorldSyncPump, WorldSyncPumpError, WorldSyncPumpReport,
    WorldSyncShutdownReceipt, WorldSyncShutdownWatchDisposition, WorldSyncShutdownWatchReceipt,
    TOPIC_WORLD_FACT,
};
pub use watch_map::{WorldWatchBinding, WorldWatchMap, WorldWatchMapError, WorldWatchProjection};

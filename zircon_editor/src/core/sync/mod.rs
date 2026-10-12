//! Editor-owned projection state for transport-neutral runtime synchronization.

#[path = "pump.rs"]
mod pump;
#[path = "watch_map.rs"]
mod watch_map;

pub use pump::{
    QualifiedWatchToken, WorldSyncPump, WorldSyncPumpError, WorldSyncPumpReport,
    WorldSyncShutdownReceipt, WorldSyncShutdownWatchDisposition, WorldSyncShutdownWatchReceipt,
    TOPIC_WORLD_FACT,
};
pub use watch_map::{WorldWatchBinding, WorldWatchMap, WorldWatchMapError, WorldWatchProjection};

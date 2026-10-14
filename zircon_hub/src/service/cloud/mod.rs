#[path = "authorization.rs"]
mod authorization;
#[path = "config.rs"]
mod config;
#[path = "manifest.rs"]
mod manifest;
#[path = "quota.rs"]
mod quota;
#[path = "retention/mod.rs"]
mod retention;
#[path = "snapshot.rs"]
mod snapshot;
#[path = "storage_recovery.rs"]
mod storage_recovery;
#[path = "store.rs"]
mod store;

pub use config::CloudConfig;
pub use manifest::{MAX_BLOB_BYTES, MAX_MANIFEST_BYTES, Manifest};
pub use quota::{Usage, usage};
pub use retention::{
    MaintenanceReport, MaintenanceRequest, RetentionOutcome, RetentionPolicy, RetentionRequest,
    RetentionState, get as retention, maintain, update as update_retention,
};
pub use snapshot::{CommitOutcome, CommitRequest, authorize_receipt, commit, head};
pub use store::{BlobStore, read_blob, upload};

#[cfg(test)]
#[path = "tests/mod.rs"]
pub(crate) mod tests;

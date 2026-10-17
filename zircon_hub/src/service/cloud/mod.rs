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
pub use manifest::{Manifest, MAX_BLOB_BYTES, MAX_MANIFEST_BYTES};
pub use quota::{usage, Usage};
pub use retention::{
    get as retention, maintain, update as update_retention, MaintenanceReport, MaintenanceRequest,
    RetentionOutcome, RetentionPolicy, RetentionRequest, RetentionState,
};
pub use snapshot::{authorize_receipt, commit, head, CommitOutcome, CommitRequest};
pub use store::{read_blob, upload, BlobStore};

#[cfg(test)]
#[path = "tests/mod.rs"]
pub(crate) mod tests;

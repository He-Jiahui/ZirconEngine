mod authorization;
mod config;
mod manifest;
mod quota;
mod retention;
mod snapshot;
mod storage_recovery;
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
pub(crate) mod tests;

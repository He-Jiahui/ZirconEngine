#[path = "../../../support/options/state/finalize.rs"]
mod finalize;
#[path = "state/storage.rs"]
mod storage;

pub(super) use storage::PendingOptionManifest;

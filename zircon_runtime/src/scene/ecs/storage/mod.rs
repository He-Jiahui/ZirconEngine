#[path = "component_remove_result.rs"]
mod component_remove_result;
#[path = "component_storage/mod.rs"]
mod component_storage;
#[path = "storage_error.rs"]
mod storage_error;

pub use component_remove_result::ComponentRemoveResult;
pub(crate) use component_storage::ComponentStorage;
pub use component_storage::ComponentStorageLocation;
pub(crate) use component_storage::PreflightedComponentInsert;
pub(crate) use component_storage::PreflightedTransferredComponentRow;
pub(crate) use component_storage::StoredComponent;
pub(crate) use component_storage::TransferredComponentRow;
pub use storage_error::StorageError;

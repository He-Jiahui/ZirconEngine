mod component_remove_result;
mod component_storage;
mod storage_error;

pub use component_remove_result::ComponentRemoveResult;
pub(crate) use component_storage::PreflightedComponentInsert;
pub(crate) use component_storage::PreflightedTransferredComponentRow;
pub(crate) use component_storage::StoredComponent;
pub(crate) use component_storage::TransferredComponentRow;
pub use component_storage::{ComponentStorage, ComponentStorageLocation};
pub use storage_error::StorageError;

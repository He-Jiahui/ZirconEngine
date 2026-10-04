mod clear_lease;
mod close_lease;
mod descriptor;
mod document_toolkit;
mod edit_lease;
mod instance_id;
mod layout;
mod registry;
mod registry_error;
mod save;
mod snapshot;

pub use clear_lease::DocumentClearLease;
pub use close_lease::DocumentCloseLease;
pub use descriptor::DocumentToolkitDescriptor;
pub use document_toolkit::{DocumentAutosavePayload, DocumentToolkit, ToolkitSaveFailure};
pub use edit_lease::DocumentEditLease;
pub use instance_id::{ToolkitInstanceId, ToolkitInstanceIdError};
pub use layout::{ToolkitArea, ToolkitAreaSlot, ToolkitLayout, ToolkitLayoutError};
pub use registry::DocumentToolkitRegistry;
pub use registry_error::ToolkitRegistryError;
pub use save::{DocumentSaveReport, SaveContextError, SaveCtx, SaveError, SaveReason};
pub(crate) use save::{DocumentSourceWritePublication, DocumentSourceWriteReceipt};
pub use snapshot::DocumentToolkitSnapshot;

#[cfg(test)]
mod tests;

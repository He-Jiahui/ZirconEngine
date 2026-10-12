#[path = "clear_lease.rs"]
mod clear_lease;
#[path = "close_lease.rs"]
mod close_lease;
#[path = "descriptor.rs"]
mod descriptor;
#[path = "document_toolkit.rs"]
mod document_toolkit;
#[path = "edit_lease.rs"]
mod edit_lease;
#[path = "instance_id.rs"]
mod instance_id;
#[path = "layout/mod.rs"]
mod layout;
#[path = "registry.rs"]
mod registry;
#[path = "registry_error.rs"]
mod registry_error;
#[path = "save/mod.rs"]
mod save;
#[path = "snapshot.rs"]
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
pub use save::{
    DocumentSaveReport, SaveContextError, SaveCtx, SaveError, SaveHookPanic, SaveReason,
};
pub(crate) use save::{DocumentSourceWritePublication, DocumentSourceWriteReceipt};
pub use snapshot::DocumentToolkitSnapshot;

#[cfg(test)]
#[path = "tests/mod.rs"]
mod tests;

mod close_lease;
mod descriptor;
mod document_toolkit;
mod instance_id;
mod layout;
mod registry;
mod registry_error;
mod save;
mod snapshot;

pub use close_lease::DocumentCloseLease;
pub use descriptor::DocumentToolkitDescriptor;
pub use document_toolkit::{DocumentAutosavePayload, DocumentToolkit, ToolkitSaveFailure};
pub use instance_id::{ToolkitInstanceId, ToolkitInstanceIdError};
pub use layout::{ToolkitArea, ToolkitAreaSlot, ToolkitLayout, ToolkitLayoutError};
pub use registry::DocumentToolkitRegistry;
pub use registry_error::ToolkitRegistryError;
pub use save::{DocumentSaveReport, SaveContextError, SaveCtx, SaveError, SaveReason};
pub use snapshot::DocumentToolkitSnapshot;

#[cfg(test)]
mod tests;

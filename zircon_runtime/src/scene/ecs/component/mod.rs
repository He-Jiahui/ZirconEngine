#[path = "id.rs"]
mod id;
#[path = "marker.rs"]
mod marker;
#[path = "registry.rs"]
mod registry;
#[path = "table_column.rs"]
mod table_column;

pub use id::ComponentId;
pub use marker::Component;
pub use registry::{ComponentDescriptor, ComponentDescriptorSource, ComponentRegistry};
pub(crate) use registry::{
    PreflightedTransferredDescriptorImports, TransferredComponentDescriptor,
};
pub(crate) use table_column::TableColumnLayout;

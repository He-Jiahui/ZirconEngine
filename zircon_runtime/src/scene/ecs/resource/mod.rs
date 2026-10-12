#[path = "id.rs"]
mod id;
#[path = "marker.rs"]
mod marker;
#[path = "registry.rs"]
mod registry;

pub use id::ResourceId;
pub use marker::Resource;
pub use registry::{ResourceDescriptor, ResourceDescriptorSource, ResourceRegistry};

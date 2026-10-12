#[path = "buffer.rs"]
mod buffer;
#[path = "../../visibility/entries.rs"]
mod entries;
#[path = "../../visibility/execute.rs"]
mod execute;
#[path = "output.rs"]
mod output;
#[path = "../../visibility/store_parts.rs"]
mod store_parts;

pub(in crate::virtual_geometry::renderer) use output::VirtualGeometryVisBuffer64PassOutput;
pub(in crate::virtual_geometry::renderer) use store_parts::VirtualGeometryVisBuffer64PassStoreParts;

pub(super) use execute::execute_virtual_geometry_visbuffer64_pass;

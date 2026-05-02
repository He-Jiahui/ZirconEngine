mod buffer;
mod entries;
mod execute;
mod output;
mod store_parts;

pub(in crate::virtual_geometry::renderer) use output::VirtualGeometryVisBuffer64PassOutput;
pub(in crate::virtual_geometry::renderer) use store_parts::VirtualGeometryVisBuffer64PassStoreParts;

pub(super) use execute::execute_virtual_geometry_visbuffer64_pass;

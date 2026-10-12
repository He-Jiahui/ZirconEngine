#[path = "buffer.rs"]
mod buffer;
#[path = "execute.rs"]
mod execute;
#[path = "output.rs"]
mod output;
#[path = "records.rs"]
mod records;
#[path = "../../raster/store_parts.rs"]
mod store_parts;

pub(in crate::virtual_geometry::renderer) use output::VirtualGeometryHardwareRasterizationPassOutput;
pub(in crate::virtual_geometry::renderer) use store_parts::VirtualGeometryHardwareRasterizationPassStoreParts;

pub(super) use execute::execute_virtual_geometry_hardware_rasterization_pass;

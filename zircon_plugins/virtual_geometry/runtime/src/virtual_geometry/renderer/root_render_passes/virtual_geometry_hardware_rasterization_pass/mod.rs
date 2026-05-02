mod buffer;
mod execute;
mod output;
mod records;
mod store_parts;

pub(in crate::virtual_geometry::renderer) use output::VirtualGeometryHardwareRasterizationPassOutput;
pub(in crate::virtual_geometry::renderer) use store_parts::VirtualGeometryHardwareRasterizationPassStoreParts;

pub(super) use execute::execute_virtual_geometry_hardware_rasterization_pass;

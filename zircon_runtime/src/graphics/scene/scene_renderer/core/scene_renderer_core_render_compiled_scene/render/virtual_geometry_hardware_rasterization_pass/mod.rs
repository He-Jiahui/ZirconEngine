mod buffer;
mod execute;
mod output;
mod records;
mod store_parts;

pub(in crate::graphics::scene::scene_renderer::core) use output::VirtualGeometryHardwareRasterizationPassOutput;
pub(in crate::graphics::scene::scene_renderer::core) use store_parts::VirtualGeometryHardwareRasterizationPassStoreParts;

pub(super) use execute::execute_virtual_geometry_hardware_rasterization_pass;

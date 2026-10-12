#[path = "shape/prefix.rs"]
mod prefix;
#[path = "shape/segment.rs"]
mod segment;

pub(super) use prefix::validate_runtime_plugin_package_coordinate_prefix;
pub(super) use segment::validate_runtime_plugin_package_coordinate_segment;

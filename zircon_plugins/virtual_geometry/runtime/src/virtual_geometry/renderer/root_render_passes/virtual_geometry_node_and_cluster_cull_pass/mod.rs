mod buffers;
mod child_decision;
mod child_worklist;
mod execute;
mod output;
mod page_requests;
mod startup_worklist;
mod store_parts;
mod traversal;
// BUG: [CR-VIRTUAL-GEOMETRY-0009] 此模块未挂载 tests/mod.rs，导致其中 child_decision、hierarchy、startup、traversal 测试不会编译或运行；证据为当前模块声明与 tests/mod.rs 的子模块声明。

pub(in crate::virtual_geometry::renderer) use output::VirtualGeometryNodeAndClusterCullPassOutput;
pub(in crate::virtual_geometry::renderer) use store_parts::VirtualGeometryNodeAndClusterCullPassStoreParts;

pub(super) use execute::execute_virtual_geometry_node_and_cluster_cull_pass;

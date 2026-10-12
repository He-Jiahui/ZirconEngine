#[path = "../../cluster_cull/buffers.rs"]
mod buffers;
#[path = "../../child_decision.rs"]
mod child_decision;
#[path = "../../child_worklist.rs"]
mod child_worklist;
#[path = "../../cluster_cull/execute.rs"]
mod execute;
#[path = "../../cluster_cull/output.rs"]
mod output;
#[path = "../../page_requests.rs"]
mod page_requests;
#[path = "../../startup_worklist.rs"]
mod startup_worklist;
#[path = "../../cluster_cull/store_parts.rs"]
mod store_parts;
#[path = "../../cluster_cull/traversal.rs"]
mod traversal;
// BUG: [CR-VIRTUAL-GEOMETRY-0009] 此模块未挂载 ../../tests/cluster_cull_mod.rs，导致其中 child_decision、hierarchy、startup、traversal 测试不会编译或运行；证据为当前模块声明与 ../../tests/cluster_cull_mod.rs 的子模块声明。

pub(in crate::virtual_geometry::renderer) use output::VirtualGeometryNodeAndClusterCullPassOutput;
pub(in crate::virtual_geometry::renderer) use store_parts::VirtualGeometryNodeAndClusterCullPassStoreParts;

pub(super) use execute::execute_virtual_geometry_node_and_cluster_cull_pass;

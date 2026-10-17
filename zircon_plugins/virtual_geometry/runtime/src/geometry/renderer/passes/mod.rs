#[path = "../../passes/packed_words.rs"]
mod packed_words;
#[path = "../../cluster_select/mod.rs"]
mod virtual_geometry_executed_cluster_selection_pass;
#[path = "../../passes/raster/mod.rs"]
mod virtual_geometry_hardware_rasterization_pass;
// TODO: [CR-VIRTUAL-GEOMETRY-0001] 核实本目录的 CPU 执行/读回投影是否计划接入产品帧，以及由谁负责激活；当前注册的 GPU executor 走独立路径，尚未找到 collect_virtual_geometry_indirect_stats 的生产调用。
#[path = "../../passes/stats/mod.rs"]
mod virtual_geometry_indirect_stats;
#[path = "../../../virtual_geometry_indirect_stats.rs"]
mod virtual_geometry_indirect_stats_store_parts;
#[path = "../../passes/cluster_cull/mod.rs"]
mod virtual_geometry_node_and_cluster_cull_pass;
#[path = "../../passes/visibility/mod.rs"]
mod virtual_geometry_visbuffer64_pass;

pub(in crate::virtual_geometry::renderer) use virtual_geometry_indirect_stats::VirtualGeometryIndirectStats;
pub(in crate::virtual_geometry::renderer) use virtual_geometry_indirect_stats_store_parts::VirtualGeometryIndirectStatsStoreParts;
pub(in crate::virtual_geometry::renderer) use virtual_geometry_node_and_cluster_cull_pass::VirtualGeometryNodeAndClusterCullPassStoreParts;

pub(crate) use virtual_geometry_node_and_cluster_cull_pass::prepare_virtual_geometry_node_and_cluster_cull;

pub(crate) use virtual_geometry_node_and_cluster_cull_pass::{
    materialize_prepared_cull, MaterializedPreparedCull,
};

mod packed_words;
mod virtual_geometry_executed_cluster_selection_pass;
mod virtual_geometry_hardware_rasterization_pass;
// TODO: [CR-VIRTUAL-GEOMETRY-0001] 核实本目录的 CPU 执行/读回投影是否计划接入产品帧，以及由谁负责激活；当前注册的 GPU executor 走独立路径，尚未找到 collect_virtual_geometry_indirect_stats 的生产调用。
mod virtual_geometry_indirect_stats;
mod virtual_geometry_indirect_stats_store_parts;
mod virtual_geometry_node_and_cluster_cull_pass;
mod virtual_geometry_visbuffer64_pass;

pub(in crate::virtual_geometry::renderer) use virtual_geometry_indirect_stats::VirtualGeometryIndirectStats;
pub(in crate::virtual_geometry::renderer) use virtual_geometry_indirect_stats_store_parts::VirtualGeometryIndirectStatsStoreParts;
pub(in crate::virtual_geometry::renderer) use virtual_geometry_node_and_cluster_cull_pass::VirtualGeometryNodeAndClusterCullPassStoreParts;
